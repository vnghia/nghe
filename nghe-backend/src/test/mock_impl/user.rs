use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use nghe_api::auth;
use o2o::o2o;
use uuid::Uuid;

use crate::orm::users;

// TODO: remove this after https://github.com/SoftbearStudios/bitcode/issues/27
#[derive(o2o)]
#[ref_into(auth::Auth<'u, 's>)]
pub struct Auth {
    #[into(~.as_str())]
    pub username: String,
    #[into(~.as_str())]
    pub salt: String,
    pub token: auth::Token,
}

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

    pub fn auth(&self) -> Auth {
        let users::Data { username, password, .. } = &self.user.data;
        let salt: String = Faker.fake();
        let token = auth::Auth::tokenize(self.mock.database().decrypt(password).unwrap(), &salt);
        Auth { username: username.to_string(), salt, token }
    }
}
