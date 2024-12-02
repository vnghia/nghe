use axum_extra::headers;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use image::EncodableLayout;
use nghe_api::auth::{self, Auth};
use uuid::Uuid;

use crate::orm::users;

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

    pub fn auth_header(&self) -> headers::Authorization<headers::authorization::Basic> {
        headers::Authorization::basic(&self.username(), &self.password())
    }

    pub fn auth_token(&self) -> Auth<'static, 'static> {
        let salt: String = Faker.fake();
        let token = auth::Token::new(self.password(), &salt);
        Auth { username: self.username().into(), salt: salt.into(), token }
    }
}
