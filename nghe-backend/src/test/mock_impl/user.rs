use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
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

    pub fn auth(&self) -> Auth<'static, 'static> {
        let users::Data { username, password, .. } = &self.user.data;
        let salt: String = Faker.fake();
        let token = auth::Token::new(self.mock.database().decrypt(password).unwrap(), &salt);
        Auth { username: username.to_string().into(), salt: salt.into(), token }
    }
}
