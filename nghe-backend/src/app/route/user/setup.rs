use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
pub use nghe_api::user::setup::{Request, Response};
use nghe_api::user::Role;
use nghe_proc_macro::handler;

use super::create;
use crate::app::state::Database;
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
                role: Role { admin: true, stream: true, download: true, share: true },
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
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_setup(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        assert!(handler(mock.database(), Faker.fake()).await.is_ok());
        assert!(handler(mock.database(), Faker.fake()).await.is_err());
    }
}
