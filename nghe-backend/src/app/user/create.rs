use diesel_async::RunQueryDsl;
pub use nghe_api::user::create::{Request, Response};
use nghe_proc_macro::handler;

use crate::app::error::Error;
use crate::app::state::Database;
use crate::orm::users;

#[handler]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { username, password, email, admin, stream, download, share, allow } = request;
    let password = database.encrypt(password);

    let user_id = diesel::insert_into(users::dsl::table)
        .values(users::New {
            password: password.into(),
            data: users::Data {
                username: username.into(),
                email: email.into(),
                role: users::Role { admin, stream, download, share },
            },
        })
        .returning(users::dsl::id)
        .get_result(&mut database.get().await?)
        .await?;

    Ok(Response { user_id })
}
