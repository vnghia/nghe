use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
pub use nghe_api::key::create::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::user_keys;

#[handler(internal = true)]
pub async fn handler(database: &Database, user_id: Uuid) -> Result<Response, Error> {
    Ok(Response {
        api_key: diesel::insert_into(user_keys::table)
            .values(user_keys::New { user_id })
            .returning(user_keys::Key::as_select())
            .get_result(&mut database.get().await?)
            .await?
            .into(),
    })
}
