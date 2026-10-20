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
macro_rules! test_roundtrip {
    ($Request:ty, $Response:ty) => {
        #[rstest::rstest]
        fn test_roundtrip() {
            use fake::{Fake, Faker};
            use $crate::auth::form::{Form, Trait};

            let request: $Request = Faker.fake();

            {
                let auth: Form = Faker.fake();
                let query = serde_html_form::to_string(&request).unwrap();
                let url = format!("{}&{}", query, serde_html_form::to_string(&auth).unwrap());
                let auth_request = serde_html_form::from_str::<
                    <$Request as $crate::http::Request>::AuthForm,
                >(&url)
                .unwrap();
                assert_eq!(auth_request.auth(), &auth);
                assert_eq!(auth_request.request(), request);
            }

            {
                assert_eq!(
                    serde_json::from_str::<$Request>(&serde_json::to_string(&request).unwrap())
                        .unwrap(),
                    request
                );
            }

            {
                let response: $Response = Faker.fake();
                assert_eq!(
                    serde_json::from_str::<$Response>(&serde_json::to_string(&response).unwrap())
                        .unwrap(),
                    response
                )
            }
        }
    };
}
