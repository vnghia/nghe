use fake::faker::internet::en::*;
use fake::{Fake, Faker};

use crate::database::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::test::CommonParams;
use crate::open_subsonic::user::test::{create_user, CreateUserParams};
use crate::utils::password::{decrypt_password, to_password_token};
use crate::Database;

impl users::User {
    pub fn fake(admin_role: Option<bool>) -> Self {
        Self {
            username: Username().fake(),
            password: Password(16..32).fake::<String>().into_bytes(),
            email: SafeEmail().fake(),
            admin_role: admin_role.unwrap_or_default(),
            ..Default::default()
        }
    }

    pub async fn create(self, db: &Database) -> Self {
        create_user(db, self.into_create_params()).await.unwrap()
    }

    pub fn into_create_params(self) -> CreateUserParams {
        let Self { username, password, email, admin_role, .. } = self;
        CreateUserParams { username, password, email, admin_role, ..Faker.fake() }
    }

    pub fn to_common_params(&self, key: &EncryptionKey) -> CommonParams {
        let decrypted_password = decrypt_password(key, &self.password).unwrap();
        let salt = Password(8..16).fake::<String>().into_bytes();
        let token = to_password_token(&decrypted_password, &salt);
        CommonParams { username: self.username.to_owned(), salt, token }
    }
}
