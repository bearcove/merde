//! The "metastack" technique allows running deeply recursive code without blowing up the stack.
//! It's an alternative to the [stacker](https://crates.io/crates/stacker) crate that uses Rust's
//! async machinery.
//!
//! metastack involves making functions into async functions: this turns them into state machines.
//! As a result, just before we're about to the overflow the stack, we put the rest of the work in
//! a "next future" thread-local, and returns `Poll::Pending`.
//!
//! This unwinds the stack all the way to the metastack landing pad, which then polls the "next future"
//! from an empty stack. That future is in turn free to schedule another "next future", and so on.
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    sync::LazyLock,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use pin_project_lite::pin_project;

type NextFuture = Pin<Box<dyn Future<Output = ()>>>;

// TODO: make this configurable? make this depend on the
// future size? 8K is not one-size-fits-all
const MINIMUM_VIABLE_FREE_STACK_SPACE: u64 = 8 * 1024;

const DUMMY_VTABLE: RawWakerVTable = RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
const DUMMY_WAKER: &Waker =
    unsafe { &Waker::from_raw(RawWaker::new(std::ptr::null(), &DUMMY_VTABLE)) };

std::thread_local! {
    pub static NEXT_FUTURE: RefCell<Option<NextFuture>> = const { RefCell::new(None) };
    pub static STACK_INFO: LazyLock<StackInfo> = LazyLock::new(StackInfo::get);
}

pub trait MetastackExt<'s>: Sized {
    type Output;

    /// Transforms a future into a future that will return `Poll::Pending` if there
    /// is not enough stack space to execute the future.
    fn with_metastack_resume_point(self) -> Pin<Box<dyn Future<Output = Self::Output> + 's>>;

    /// Sets up a landing pad to catch `Poll::Pending` returns and run the next
    /// scheduled future on a slightly emptier stack.
    fn run_sync_with_metastack(self) -> Self::Output;

    /// Sets up a landing pad to catch `Poll::Pending` returns and run the next
    /// scheduled future on a slightly emptier stack — but also supports yielding
    /// to the async runtime.
    fn run_async_with_metastack(self) -> impl Future<Output = Self::Output>;
}

impl<'s, F> MetastackExt<'s> for F
where
    F: Future + 's,
{
    type Output = F::Output;

    fn with_metastack_resume_point(self) -> Pin<Box<dyn Future<Output = Self::Output> + 's>> {
        with_metastack_resume_point(self)
    }

    fn run_sync_with_metastack(self) -> Self::Output {
        let mut cx = Context::from_waker(DUMMY_WAKER);
        let mut first_fut = std::pin::pin!(self);

        match first_fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => res,
            _ => {
                // oh boy. okay.
                let mut metastack = vec![];

                'crimes: loop {
                    let mut fut = NEXT_FUTURE
                        .with_borrow_mut(|next_fut| next_fut.take())
                        .expect("NEXT_FUTURE must've been set before returning Poll::Pending");
                    match Pin::new(&mut fut).poll(&mut cx) {
                        Poll::Ready(_) => break 'crimes,
                        Poll::Pending => {
                            metastack.push(fut);
                        }
                    }
                }

                while let Some(mut fut) = metastack.pop() {
                    match Pin::new(&mut fut).poll(&mut cx) {
                        Poll::Ready(_) => {
                            // cool let's keep going
                        }
                        Poll::Pending => {
                            unreachable!("I'm sorry you really only get to ask for more stack once")
                        }
                    }
                }

                match first_fut.poll(&mut cx) {
                    Poll::Ready(res) => res,
                    Poll::Pending => {
                        unreachable!("Like I said, you really only get to ask for more stack once")
                    }
                }
            }
        }
    }

    fn run_async_with_metastack(self) -> impl Future<Output = Self::Output> {
        MetastackFutureAdapter {
            future: self,
            // perf note: empty vecs don't allocate yet, so `Option<Vec<T>>` is pointless
            recursions: Default::default(),
        }
    }
}

pin_project! {
    struct MetastackFutureAdapter<F: Future> {
        #[pin]
        future: F,
        recursions: Vec<NextFuture>,
    }
}

impl<F> Future for MetastackFutureAdapter<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        // goto considered harmful? just use a loop 😎
        loop {
            if let Some(mut fut) = this.recursions.pop() {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(_) => {
                        // cool, let's keep unwinding the metastack
                        continue;
                    }
                    Poll::Pending => {
                        // are we suspending for I/O, or did we almost run out of stack space?
                        match NEXT_FUTURE.with_borrow_mut(|next_future| next_future.take()) {
                            Some(next_fut) => {
                                // needs more stack space, alrighty then.
                                this.recursions.push(fut);
                                this.recursions.push(next_fut);
                                continue;
                            }
                            None => {
                                // just
                            }
                        }
                    }
                }
            }

            // ran out of recursions (or it's the first poll), let's poll the first future
            match this.future.as_mut().poll(cx) {
                Poll::Ready(res) => {
                    // wow we did it!
                    return Poll::Ready(res);
                }
                Poll::Pending => {
                    // are we suspending for I/O, or did we almost run out of stack space?
                    match NEXT_FUTURE.with_borrow_mut(|next_future| next_future.take()) {
                        Some(next_fut) => {
                            // needs more stack space, alrighty then.
                            this.recursions.push(next_fut);
                            continue;
                        }
                        None => {
                            // just
                        }
                    }
                }
            }
        }
    }
}

/// Transforms a future into a future that will return `Poll::Pending` if there
/// is not enough stack space to execute the future.
///
/// This relies on the current async stack being invoked via `run_with_infinite_stack`
pub fn with_metastack_resume_point<'s, F>(fut: F) -> Pin<Box<dyn Future<Output = F::Output> + 's>>
where
    F: Future + 's,
{
    Box::pin(async move {
        if STACK_INFO.with(|si| si.left()) >= MINIMUM_VIABLE_FREE_STACK_SPACE {
            // no need for any special handling
            return fut.await;
        }

        // this looks like it's on the stack, but it's not! because we're
        // in a boxed future — we'll be pinned somewhere in memory before
        // we get a chance to take a reference to it.
        let mut result: Option<F::Output> = None;

        // make a future that will actually assign the result
        let assign_fut: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async {
            result = Some(fut.await);
        });

        // # Safety: this isn't actually 'static, it's "valid for the synchronous
        // call to deserialize".
        // todo: make sure that this is actually the case by handling panics and
        // clearing thread-locals.
        let assign_fut: Pin<Box<dyn Future<Output = ()> + 'static>> =
            unsafe { std::mem::transmute(assign_fut) };

        NEXT_FUTURE.with_borrow_mut(|next_future| *next_future = Some(assign_fut));
        ReturnPendingOnce::new().await;
        result.unwrap()
    })
}

/// A future that returns `Poll::Pending` once, and then `Poll::Ready`
struct ReturnPendingOnce {
    polled: bool,
}

impl ReturnPendingOnce {
    fn new() -> Self {
        Self { polled: false }
    }
}

impl Future for ReturnPendingOnce {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.polled {
            Poll::Ready(())
        } else {
            this.polled = true;
            Poll::Pending
        }
    }
}

/// Stack information — we always assume the stack grows down (e.g. the more we
/// allocate, the "lower" the address).
pub struct StackInfo {
    /// The highest possible address of the stack
    highest_address: u64,

    /// The size of the stack
    size: u64,
}

impl StackInfo {
    pub fn get() -> Self {
        #[cfg(target_os = "macos")]
        unsafe {
            use std::os::raw::c_void;

            extern "C" {
                fn pthread_get_stackaddr_np(thread: u64) -> *mut c_void;
                fn pthread_get_stacksize_np(thread: u64) -> usize;
                fn pthread_self() -> u64;
            }

            let thread = pthread_self();
            let stack_addr = pthread_get_stackaddr_np(thread) as u64;
            let size = pthread_get_stacksize_np(thread) as u64;

            Self {
                highest_address: stack_addr,
                size,
            }
        }

        #[cfg(target_os = "linux")]
        {
            unsafe {
                use std::mem;
                use std::os::raw::c_void;

                extern "C" {
                    fn pthread_attr_init(attr: *mut pthread_attr_t) -> i32;
                    fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> i32;
                    fn pthread_getattr_np(thread: pthread_t, attr: *mut pthread_attr_t) -> i32;
                    fn pthread_attr_getstack(
                        attr: *const pthread_attr_t,
                        stackaddr: *mut *mut c_void,
                        stacksize: *mut usize,
                    ) -> i32;
                    fn pthread_self() -> pthread_t;
                }

                #[repr(C)]
                #[allow(non_camel_case_types)]
                struct pthread_attr_t {
                    __size: [u64; 7],
                }

                #[allow(non_camel_case_types)]
                type pthread_t = usize;

                let mut attr: pthread_attr_t = mem::zeroed();
                let mut lowest_address: *mut c_void = std::ptr::null_mut();
                let mut size: usize = 0;

                pthread_attr_init(&mut attr);
                pthread_getattr_np(pthread_self(), &mut attr);
                pthread_attr_getstack(&attr, &mut lowest_address, &mut size);
                pthread_attr_destroy(&mut attr);

                let size = size as u64;
                let highest_address = lowest_address as u64 + size;

                Self {
                    highest_address,
                    size,
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            unsafe {
                use std::mem;
                use std::ptr;

                #[repr(C)]
                struct MEMORY_BASIC_INFORMATION {
                    base_address: *mut std::ffi::c_void,
                    allocation_base: *mut std::ffi::c_void,
                    allocation_protect: u32,
                    region_size: usize,
                    state: u32,
                    protect: u32,
                    type_: u32,
                }

                extern "system" {
                    fn VirtualQuery(
                        lp_address: *const std::ffi::c_void,
                        lp_buffer: *mut MEMORY_BASIC_INFORMATION,
                        dw_length: usize,
                    ) -> usize;
                }

                let mut stack_info: MEMORY_BASIC_INFORMATION = mem::zeroed();
                let stack_pointer: *const std::ffi::c_void = ptr::null();

                VirtualQuery(
                    stack_pointer,
                    &mut stack_info,
                    mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );

                Self {
                    stack_base: stack_info.allocation_base as u64,
                    stack_size: stack_info.region_size as u64,
                }
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        panic!("Unsupported platform")
    }

    /// How much stack space is left?
    pub fn left(&self) -> u64 {
        let stack_var: u64 = 0;
        let stack_top = &stack_var as *const u64;

        self.size
            .checked_sub(
                self.highest_address
                    .checked_sub(stack_top as u64)
                    .expect("we assume the stack grows down"),
            )
            .expect("we assume we haven't exhausted the whole stack")
    }
}
