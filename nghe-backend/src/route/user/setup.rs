use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use nghe_api::user::Role;
pub use nghe_api::user::setup::{Request, Response};
use nghe_proc_macro::handler;

use super::create;
use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

#[handler(need_auth = false, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    if users::table.count().first::<i64>(&mut database.get().await?).await? > 0 {
        error::Kind::Forbidden.into()
    } else {
        let Request { username, password, email } = request;
        create::handler(
            database,
            create::Request { username, password, email, role: Role { admin: true }, allow: false },
        )
        .await?;
        Ok(Response)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

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
