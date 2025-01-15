use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
pub use nghe_api::user::list::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database) -> Result<Response, Error> {
    Ok(Response {
        users: users::table
            .select(users::Info::as_select())
            .get_results(&mut database.get().await?)
            .await?
            .into_iter()
            .map(users::Info::into)
            .collect(),
    })
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        mock.add_user().call().await.add_user().call().await;
        let users = handler(mock.database()).await.unwrap().users;
        assert_eq!(users.len(), 2);
    }
}
