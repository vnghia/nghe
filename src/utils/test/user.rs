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
    pub basic: users::BasicUser<'static>,
    pub id: Uuid,
    pub password: Vec<u8>,
}

impl User {
    pub fn fake(role: Option<users::Role>) -> Self {
        Self {
            basic: users::BasicUser {
                username: Username().fake::<String>().into(),
                role: role.unwrap_or_default(),
            },
            id: Faker.fake(),
            password: Password(16..32).fake::<String>().into_bytes(),
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
        Self { username: value.basic.username.to_string(), salt, token }
    }
}

impl From<User> for CreateUserParams {
    fn from(value: User) -> Self {
        let User { basic, password, .. } = value;
        CreateUserParams {
            username: basic.username.to_string(),
            password: hex::encode(password).into_bytes(),
            email: SafeEmail().fake(),
            admin_role: basic.role.admin_role,
            stream_role: basic.role.stream_role,
            download_role: basic.role.download_role,
            share_role: basic.role.share_role,
        }
    }
}
