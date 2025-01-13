use super::context;

pub trait Toast {
    type Out;

    fn toast(self) -> Option<Self::Out>;
}

impl<T> Toast for Result<T, anyhow::Error> {
    type Out = T;

    fn toast(self) -> Option<Self::Out> {
        match self {
            Ok(t) => {
                context::Error::clear();
                Some(t)
            }
            Err(e) => {
                leptos::logging::error!("{:?}", e);
                context::Error::set(e.to_string());
                None
            }
        }
    }
}
