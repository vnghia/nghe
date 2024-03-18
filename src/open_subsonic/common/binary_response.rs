use std::convert::Infallible;

use crossfire::{
    channel::{MPSCShared, Stream},
    mpsc,
};
use futures::StreamExt;

pub struct StreamResponse<S: MPSCShared>(pub Stream<Vec<u8>, mpsc::RxFuture<Vec<u8>, S>>);

impl<S: MPSCShared> StreamResponse<S> {
    pub fn new(rx: mpsc::RxFuture<Vec<u8>, S>) -> Self {
        Self(rx.into_stream())
    }
}

impl<S: MPSCShared> futures::Stream for StreamResponse<S> {
    type Item = Result<Vec<u8>, Infallible>;

    // TODO: remove this when axum::body does not require TryStream anymore.
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx).map(|t| t.map(Ok))
    }
}
