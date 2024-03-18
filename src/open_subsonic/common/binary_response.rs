use std::convert::Infallible;

use futures::StreamExt;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;

pub struct StreamResponse(pub ReceiverStream<Vec<u8>>);

impl StreamResponse {
    pub fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self(ReceiverStream::new(rx))
    }
}

impl futures::Stream for StreamResponse {
    type Item = Result<Vec<u8>, Infallible>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx).map(|t| t.map(Ok))
    }
}
