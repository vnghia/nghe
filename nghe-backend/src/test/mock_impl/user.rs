use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use image::EncodableLayout;
use nghe_api::auth;
use uuid::Uuid;

use crate::http::extract::auth::header::{BaiscAuthorization, BearerAuthorization};
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

    pub fn username(&self) -> String {
        self.user.data.info.username.to_string()
    }

    fn password(&self) -> String {
        String::from_utf8(self.mock.database().decrypt(self.user.data.password.as_bytes()).unwrap())
            .unwrap()
    }

    pub async fn api_key(&self) -> auth::ApiKey {
        key::create::handler(self.mock.database(), key::create::Request {
            username: self.username(),
            password: self.password(),
            client: Faker.fake::<String>(),
        })
        .await
        .unwrap()
        .api_key
    }

    pub async fn auth_bearer(&self) -> BearerAuthorization {
        BearerAuthorization::bearer(&self.api_key().await.api_key.to_string()).unwrap()
    }

    pub fn auth_basic(&self) -> BaiscAuthorization {
        BaiscAuthorization::basic(&self.username(), &self.password())
    }

    // use_token: None -> use ApiKey
    // use_token: Some(true) -> use Token
    // use_token: Some(false) -> use Password
    pub async fn auth_form(
        &self,
        use_token: Option<bool>,
    ) -> auth::Form<'static, 'static, 'static, 'static> {
        if let Some(use_token) = use_token {
            let username = self.username().into();
            let client = Faker.fake::<String>().into();
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
            self.api_key().await.api_key.into()
        }
    }
}
