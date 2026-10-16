#[macro_export]
macro_rules! test_deserialize_form {
    ($url:ident, $request:ident, $Request:ty) => {
        assert_eq!(serde_html_form::from_str::<Request>($url).ok(), $request);

        if let Some(request) = $request {
            use fake::{Fake, Faker};
            use $crate::auth::form::Trait;
            use $crate::auth::username::Auth;

            let username: String = Faker.fake();
            let client: String = Faker.fake();
            let password_: String = Faker.fake();

            let url = format!("{}&u={username}&c={client}&p={password_}", $url);
            {
                let auth_request =
                    serde_html_form::from_str::<<Request as $crate::http::Request>::AuthForm>(&url)
                        .unwrap();
                let auth = auth_request.auth().clone().try_as_username().unwrap();

                assert_eq!(auth.username, username);
                assert_eq!(auth.client, client);

                match auth.auth {
                    Auth::Token(_) => panic!("This can not be a token"),
                    Auth::Password { password } => {
                        assert_eq!(password, password_);
                    }
                }

                assert_eq!(auth_request.request(), request);
            }
        }
    };
}
