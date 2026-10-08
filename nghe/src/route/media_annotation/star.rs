pub use nghe_api::media_annotation::star::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{star_albums, star_artists, star_songs};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    if !request.song_ids.is_empty() {
        star_songs::Upsert::upserts(database, user_id, &request.song_ids).await?;
    }
    if !request.album_ids.is_empty() {
        star_albums::Upsert::upserts(database, user_id, &request.album_ids).await?;
    }
    if !request.artist_ids.is_empty() {
        star_artists::Upsert::upserts(database, user_id, &request.artist_ids).await?;
    }
    Ok(Response)
}
