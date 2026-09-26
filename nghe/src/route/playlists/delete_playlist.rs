use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::playlists::delete_playlist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{playlist, playlists};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let playlist_id = request.id;
    playlist::permission::check_write(database, playlist_id, user_id, true).await?;
    diesel::delete(playlists::table)
        .filter(playlists::id.eq(playlist_id))
        .execute(&mut database.get().await?)
        .await?;
    Ok(Response)
}
