use serde::Deserialize;

use super::error::{ClientError, LastfmError};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum LastfmResponse<T> {
    Ok(T),
    Err(LastfmError),
}

impl<T> From<LastfmResponse<T>> for Result<T, ClientError> {
    fn from(value: LastfmResponse<T>) -> Self {
        match value {
            LastfmResponse::Ok(ok) => Self::Ok(ok),
            LastfmResponse::Err(err) => Self::Err(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Dummy, Fake, Faker};
    use serde_json::json;

    use super::*;
    use crate::error::LastfmErrorCode;

    #[derive(Debug, Deserialize, Dummy, PartialEq, Eq)]
    struct TestResponse {
        a: i32,
        b: String,
    }

    #[test]
    fn test_der_response_ok() {
        let test = TestResponse { ..Faker.fake() };
        let der_response: LastfmResponse<TestResponse> = serde_json::from_value(json!({
            "a": test.a,
            "b": test.b.clone(),
        }))
        .unwrap();
        let response = LastfmResponse::Ok(test);
        assert_eq!(der_response, response);
    }

    #[test]
    fn test_der_response_error() {
        let der_response: LastfmResponse<TestResponse> = serde_json::from_value(json!({
            "code": 2,
            "message": "message",
        }))
        .unwrap();
        let response = LastfmResponse::Err(LastfmError {
            code: LastfmErrorCode::InvalidService,
            message: "message".into(),
        });
        assert_eq!(der_response, response);
    }
}
