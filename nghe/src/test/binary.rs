use axum::http::{HeaderName, HeaderValue};
use axum_extra::headers;
use strum::{EnumString, IntoStaticStr};

static BINARY_STATUS: HeaderName = HeaderName::from_static("x-binary-status");

#[derive(Debug, Clone, Copy, EnumString, IntoStaticStr, PartialEq, Eq, PartialOrd, Ord)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    NoCache,
    WithCache,
    ServeCachedOutput,
    UseCachedOutput,
}

pub struct Header(pub Status);

impl headers::Header for Header {
    fn name() -> &'static HeaderName {
        &BINARY_STATUS
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        Ok(Self(
            values
                .next()
                .ok_or_else(headers::Error::invalid)?
                .to_str()
                .map_err(|_| headers::Error::invalid())?
                .parse()
                .map_err(|_| headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_static(self.0.into())));
    }
}
