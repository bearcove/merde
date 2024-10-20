mod cowstr;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;

pub use cowstr::CowStr;

mod cowbytes;
pub use cowbytes::CowBytes;

mod array;
pub use array::Array;

mod map;
pub use map::Map;

mod error;
pub use error::MerdeError;
pub use error::ValueType;

mod into_static;
pub use into_static::IntoStatic;

mod with_lifetime;
pub use with_lifetime::WithLifetime;

mod value;
pub use value::Value;

mod deserialize;
pub use deserialize::ArrayStart;
pub use deserialize::Deserialize;
pub use deserialize::DeserializeOwned;
pub use deserialize::Deserializer;
pub use deserialize::Event;
pub use deserialize::EventType;

type BoxFuture = Pin<Box<dyn Future<Output = ()>>>;

std::thread_local! {
    pub static NEXT_FUTURE: RefCell<Option<BoxFuture>> = const { RefCell::new(None) };
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

rubicon::compatibility_check! {
    ("merde_core_pkg_version", env!("CARGO_PKG_VERSION")),

    #[cfg(feature = "compact_str")]
    ("compact_str", "enabled")

    #[cfg(feature = "compact_bytes")]
    ("compact_bytes", "enabled")
}
