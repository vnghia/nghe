use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use nghe_api::auth;
pub use nghe_api::key::create::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::http::extract::auth::Authentication;
use crate::orm::user_keys;

#[handler(need_auth = false, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { username, password, client } = request;
    let user_id =
        auth::Username { username: username.into(), client: client.into(), auth: password.into() }
            .authenticated(database)
            .await?
            .id;
    Ok(Response {
        api_key: diesel::insert_into(user_keys::table)
            .values(user_keys::New { user_id })
            .returning(user_keys::Key::as_select())
            .get_result(&mut database.get().await?)
            .await?
            .into(),
    })
}
