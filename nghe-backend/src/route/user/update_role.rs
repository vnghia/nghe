use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::user::update_role::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { id, role } = request;
    diesel::update(users::table)
        .filter(users::id.eq(id))
        .set(users::Role::from(role))
        .execute(&mut database.get().await?)
        .await?;
    Ok(Response)
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
        let user = mock.add_user().role(users::Role { admin: false }).call().await.user(0).await;
        let role = users::Role { admin: true };
        handler(mock.database(), Request { id: user.id(), role: role.into() }).await.unwrap();
        let user = mock.user(0).await;
        assert_eq!(user.role(), role);
    }
}
