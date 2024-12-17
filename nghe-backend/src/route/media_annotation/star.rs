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
    if let Some(ref song_ids) = request.song_ids {
        star_songs::Upsert::upserts(database, user_id, song_ids).await?;
    }
    if let Some(ref album_ids) = request.album_ids {
        star_albums::Upsert::upserts(database, user_id, album_ids).await?;
    }
    if let Some(ref artist_ids) = request.artist_ids {
        star_artists::Upsert::upserts(database, user_id, artist_ids).await?;
    }
    Ok(Response)
}
