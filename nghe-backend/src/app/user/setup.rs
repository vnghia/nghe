#![allow(clippy::unused_async)]

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
pub use nghe_api::user::setup::{Request, Response};
use nghe_proc_macro::handler;

use crate::app::state::Database;
use crate::app::user::create;
use crate::orm::users;
use crate::Error;

#[handler]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    if users::table.count().first::<i64>(&mut database.get().await?).await? > 0 {
        Err(Error::Unauthorized("Could not access setup endpoint when there is already one user"))
    } else {
        let Request { username, password, email } = request;
        create::handler(
            database,
            create::Request {
                username,
                password,
                email,
                admin: true,
                stream: true,
                download: true,
                share: true,
                allow: false,
            },
        )
        .await?;
        Ok(Response)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::test::Mock;

    #[tokio::test]
    async fn test_setup_no_user() {
        let mock = Mock::new().await;
        assert!(handler(mock.database(), Faker.fake()).await.is_ok());
    }

    #[tokio::test]
    async fn test_setup_with_user() {
        let mock = Mock::new().await;
        assert!(handler(mock.database(), Faker.fake()).await.is_ok());
        assert!(handler(mock.database(), Faker.fake()).await.is_err());
    }
}
