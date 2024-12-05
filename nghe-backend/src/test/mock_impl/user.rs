use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use image::EncodableLayout;
use nghe_api::auth;
use uuid::Uuid;

use crate::http::extract::auth::header::BaiscAuthorization;
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

    pub fn auth_header(&self) -> BaiscAuthorization {
        BaiscAuthorization::basic(&self.username(), &self.password())
    }

    pub fn auth_form(&self) -> auth::Form<'static, 'static> {
        let salt: String = Faker.fake();
        let token = auth::Token::new(self.password(), &salt);
        auth::Form::Token(auth::token::Auth {
            username: self.username().into(),
            salt: salt.into(),
            token,
        })
    }
}
