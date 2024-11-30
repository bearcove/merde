// adapted from <https://github.com/rvolosatovs/send-future/blob/main/src/lib.rs>

//! This crate provides [`SendFuture::send`] workaround for compiler bug
//! [https://github.com/rust-lang/rust/issues/96865](https://github.com/rust-lang/rust/issues/96865)
//!
//! See documentation of [`SendFuture`] trait for example usage

/// This trait is used as a workaround for compiler bug
/// [https://github.com/rust-lang/rust/issues/96865](https://github.com/rust-lang/rust/issues/96865)
///
/// Compilation of code calling async methods defined using `impl` syntax within [`Send`] async functions
/// may fail with hard-to-debug errors.
///
/// The following fails to compile with rustc 1.78.0:
///
/// ```compile_fail
/// trait X {
///     fn test<Y>(&self, x: impl AsRef<[Y]>) -> impl core::future::Future<Output = ()> + Send
///     where
///         Y: AsRef<str>;
/// }
///
/// fn call_test(x: impl X + Send + Sync) -> impl core::future::Future<Output = ()> + Send {
///     async move { x.test(["test"]).await }
/// }
/// ```
///
/// ```text
/// error: implementation of `AsRef` is not general enough
///   --> src/lib.rs:66:9
///    |
/// 66 |         async move { x.test(["test"]).await }
///    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ implementation of `AsRef` is not general enough
///    |
///    = note: `[&'0 str; 1]` must implement `AsRef<[&'1 str]>`, for any two lifetimes `'0` and `'1`...
///    = note: ...but it actually implements `AsRef<[&str]>`
/// ```
///
/// The fix is to call [`send`](SendFuture::send) provided by this trait on the future before awaiting:
/// ```
/// use send_future::SendFuture as _;
///
/// trait X {
///     fn test<Y>(&self, x: impl AsRef<[Y]>) -> impl core::future::Future<Output = ()> + Send
///     where
///         Y: AsRef<str>;
/// }
///
/// fn call_test(x: impl X + Send + Sync) -> impl core::future::Future<Output = ()> + Send {
///     async move { x.test(["test"]).send().await }
/// }
/// ```
pub trait SendFuture<'s>: core::future::Future + 's {
    fn send(self) -> impl core::future::Future<Output = Self::Output> + Send + 's
    where
        Self: Sized + Send,
    {
        self
    }
}

impl<'s, T: core::future::Future + 's> SendFuture<'s> for T {}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use super::SendFuture as _;

    trait X {
        fn test<Y>(&self, x: impl AsRef<[Y]>) -> impl core::future::Future<Output = ()> + Send
        where
            Y: AsRef<str>;
    }

    fn call_test(x: impl X + Send + Sync) -> impl core::future::Future<Output = ()> + Send {
        async move { x.test(["test"]).send().await }
    }
}
