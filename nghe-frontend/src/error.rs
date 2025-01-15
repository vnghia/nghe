use std::sync::Arc;

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error(transparent)]
    GlooNet(#[from] Arc<gloo_net::Error>),
    #[error(transparent)]
    Http(#[from] Http),
}

#[derive(Debug, thiserror::Error, Clone)]
#[error("{code} {text}")]
pub struct Http {
    pub code: u16,
    pub text: String,
}

impl From<gloo_net::Error> for Error {
    fn from(value: gloo_net::Error) -> Self {
        Arc::new(value).into()
    }
}
