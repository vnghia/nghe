use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::user::delete::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    diesel::delete(users::table)
        .filter(users::id.eq(request.user_id))
        .execute(&mut database.get().await?)
        .await?;
    Ok(Response)
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::route::user::list;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(2, 0)]
        mock: Mock,
    ) {
        let user_id_1 = mock.user_id(0).await;
        let user_id_2 = mock.user_id(1).await;
        handler(mock.database(), Request { user_id: user_id_1 }).await.unwrap();
        let users = list::handler(mock.database()).await.unwrap().users;
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, user_id_2);
    }
}
