use fake::faker::internet::en::*;
use fake::{Fake, Faker};
use nghe_types::params::{to_password_token, CommonParams};
use nghe_types::user::create::CreateUserParams;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::user::test::create_user;
use crate::Database;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: Vec<u8>,
    pub role: users::Role,
}

impl User {
    pub fn fake(role: Option<users::Role>) -> Self {
        Self {
            id: Faker.fake(),
            username: Username().fake(),
            password: Password(16..32).fake::<String>().into_bytes(),
            role: role.unwrap_or_default(),
        }
    }

    pub async fn create(self, db: &Database) -> Self {
        let id = create_user(db, &self.clone().into()).await.unwrap();
        Self { id, ..self }
    }
}

impl From<&User> for CommonParams {
    fn from(value: &User) -> Self {
        let salt = Password(8..16).fake::<String>();
        let token = to_password_token(&value.password, &salt);
        Self { username: value.username.to_string(), salt, token }
    }
}

impl From<User> for CreateUserParams {
    fn from(value: User) -> Self {
        let User { username, password, role, .. } = value;
        CreateUserParams { username, password, email: SafeEmail().fake(), role: role.into() }
    }
}
