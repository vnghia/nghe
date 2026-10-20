#[macro_export]
macro_rules! test_deserialize_form {
    ($url:ident, $request:ident, $Request:ty) => {
        assert_eq!(serde_html_form::from_str::<$Request>($url).ok(), $request);

        if let Some(request) = $request {
            use fake::{Fake, Faker};
            use $crate::auth::form::{Form, Trait};

            let auth: Form = Faker.fake();
            let url = format!("{}&{}", $url, serde_html_form::to_string(&auth).unwrap());
            let auth_request =
                serde_html_form::from_str::<<$Request as $crate::http::Request>::AuthForm>(&url)
                    .unwrap();
            assert_eq!(auth_request.auth(), &auth);
            assert_eq!(auth_request.request(), request);
        }
    };
}

#[macro_export]
macro_rules! test_roundtrip_request_json {
    ($Request:ty) => {
        #[rstest::rstest]
        fn test_roundtrip_request_json() {
            use fake::{Fake, Faker};

            let request: $Request = Faker.fake();
            let serialized = serde_json::to_string(&request).unwrap();
            assert_eq!(serde_json::from_str::<$Request>(&serialized).expect(&serialized), request);
        }
    };
}

#[macro_export]
macro_rules! test_roundtrip_request {
    ($Request:ty) => {
        #[rstest::rstest]
        fn test_roundtrip_request_form() {
            use fake::{Fake, Faker};
            use $crate::auth::form::{Form, Trait};

            let request: $Request = Faker.fake();

            let auth: Form = Faker.fake();
            let query = serde_html_form::to_string(&request).unwrap();
            let url = format!("{}&{}", query, serde_html_form::to_string(&auth).unwrap());
            let auth_request =
                serde_html_form::from_str::<<$Request as $crate::http::Request>::AuthForm>(&url)
                    .expect(&url);
            assert_eq!(auth_request.auth(), &auth);
            assert_eq!(auth_request.request(), request);
        }

        $crate::test_roundtrip_request_json!($Request);
    };
}

#[macro_export]
macro_rules! test_roundtrip_response {
    ($Response:ty) => {
        #[rstest::rstest]
        fn test_roundtrip_response_json() {
            use fake::{Fake, Faker};

            let response: $Response = Faker.fake();
            let serialized = serde_json::to_string(&response).unwrap();
            assert_eq!(serde_json::from_str::<$Response>(&serialized).expect(&serialized), response)
        }
    };
}

#[macro_export]
macro_rules! test_roundtrip_json {
    ($Request:ty, $Response:ty) => {
        $crate::test_roundtrip_request_json!($Request);
        $crate::test_roundtrip_response!($Response);
    };
}

#[macro_export]
macro_rules! test_roundtrip {
    ($Request:ty, $Response:ty) => {
        $crate::test_roundtrip_request!($Request);
        $crate::test_roundtrip_response!($Response);
    };
}
