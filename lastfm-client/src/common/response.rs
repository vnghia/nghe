use serde::Deserialize;

use super::error::{ClientError, LastFmError};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum LastFmResponse<T> {
    Ok(T),
    Err(LastFmError),
}

impl<T> From<LastFmResponse<T>> for Result<T, ClientError> {
    fn from(value: LastFmResponse<T>) -> Self {
        match value {
            LastFmResponse::Ok(ok) => Self::Ok(ok),
            LastFmResponse::Err(err) => Self::Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Dummy, Fake, Faker};
    use serde_json::json;

    use super::*;
    use crate::error::LastFmErrorCode;

    #[derive(Debug, Deserialize, Dummy, PartialEq, Eq)]
    struct TestResponse {
        a: i32,
        b: String,
    }

    #[test]
    fn test_der_response_ok() {
        let test = TestResponse { ..Faker.fake() };
        let der_response: LastFmResponse<TestResponse> = serde_json::from_value(json!({
            "a": test.a,
            "b": test.b.clone(),
        }))
        .unwrap();
        let response = LastFmResponse::Ok(test);
        assert_eq!(der_response, response);
    }

    #[test]
    fn test_der_response_error() {
        let der_response: LastFmResponse<TestResponse> = serde_json::from_value(json!({
            "code": 2,
            "message": "message",
        }))
        .unwrap();
        let response = LastFmResponse::Err(LastFmError {
            code: LastFmErrorCode::InvalidService,
            message: "message".into(),
        });
        assert_eq!(der_response, response);
    }
}
