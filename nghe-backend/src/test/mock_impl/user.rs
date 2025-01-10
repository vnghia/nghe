use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use image::EncodableLayout;
use nghe_api::auth;
use uuid::Uuid;

use crate::http::extract::auth::header::BaiscAuthorization;
use crate::orm::users;
use crate::route::key;

pub struct Mock<'a> {
    mock: &'a super::Mock,
    pub user: users::User<'static>,
}

impl<'a> Mock<'a> {
    pub async fn new(mock: &'a super::Mock, index: usize) -> Self {
        Self {
            mock,
            user: users::table
                .select(users::User::as_select())
                .order_by(users::created_at)
                .offset(index.try_into().unwrap())
                .first(&mut mock.get().await)
                .await
                .unwrap(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    fn username(&self) -> String {
        self.user.data.username.to_string()
    }

    fn password(&self) -> String {
        String::from_utf8(self.mock.database().decrypt(self.user.data.password.as_bytes()).unwrap())
            .unwrap()
    }

    pub fn auth_header(&self) -> BaiscAuthorization {
        BaiscAuthorization::basic(&self.username(), &self.password())
    }

    // use_token: None -> use ApiKey
    // use_token: Some(true) -> use Token
    // use_token: Some(false) -> use Password
    pub async fn auth_form(
        &self,
        use_token: Option<bool>,
    ) -> auth::Form<'static, 'static, 'static, 'static> {
        let username = self.username();
        let password = self.password();
        let client = Faker.fake::<String>();

        if let Some(use_token) = use_token {
            let username = username.into();
            let client = client.into();
            if use_token {
                let salt: String = Faker.fake();
                let token = auth::username::Token::new(self.password(), &salt);
                auth::Username {
                    username,
                    client,
                    auth: auth::username::token::Auth { salt: salt.into(), token }.into(),
                }
                .into()
            } else {
                auth::Username { username, client, auth: self.password().into() }.into()
            }
        } else {
            key::create::handler(self.mock.database(), key::create::Request {
                username,
                password,
                client,
            })
            .await
            .unwrap()
            .api_key
            .api_key
            .into()
        }
    }
}
