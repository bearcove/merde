use std::future::Future;

use crate::{CowStr, Event, MerdeError};

fn _assert_cow_str_covariant<'s>(cs: CowStr<'static>) -> CowStr<'s> {
    cs
}

fn _assert_event_covariant<'s>(e: Event<'static>) -> Event<'s> {
    e
}

fn _assert_merde_error_covariant<'s>(me: MerdeError<'static>) -> MerdeError<'s> {
    me
}

fn _assert_event_result_covariant<'s>(
    er: Result<Event<'static>, MerdeError<'static>>,
) -> Result<Event<'s>, MerdeError<'s>> {
    er
}

#[allow(clippy::manual_async_fn)]
fn _assert_future_event_covariant<'s>(
    f: impl Future<Output = Result<Event<'static>, MerdeError<'static>>> + 'static,
) -> impl Future<Output = Result<Event<'s>, MerdeError<'s>>> {
    // see <https://github.com/rust-lang/rust/issues/133676>
    #[allow(clippy::redundant_async_block)]
    async move {
        f.await
    }
}
