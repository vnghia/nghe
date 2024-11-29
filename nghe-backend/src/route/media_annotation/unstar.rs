use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::media_annotation::unstar::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{star_albums, star_artists, star_songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    if let Some(ref song_ids) = request.song_ids {
        diesel::delete(star_songs::table)
            .filter(star_songs::user_id.eq(user_id))
            .filter(star_songs::song_id.eq_any(song_ids))
            .execute(&mut database.get().await?)
            .await?;
    }
    if let Some(ref album_ids) = request.album_ids {
        diesel::delete(star_albums::table)
            .filter(star_albums::user_id.eq(user_id))
            .filter(star_albums::album_id.eq_any(album_ids))
            .execute(&mut database.get().await?)
            .await?;
    }
    if let Some(ref artist_ids) = request.artist_ids {
        diesel::delete(star_artists::table)
            .filter(star_artists::user_id.eq(user_id))
            .filter(star_artists::artist_id.eq_any(artist_ids))
            .execute(&mut database.get().await?)
            .await?;
    }
    Ok(Response)
}
