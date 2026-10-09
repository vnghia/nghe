use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::route::media_annotation::unstar::{Request, Response};
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
        diesel::delete(star_songs::table)
            .filter(star_songs::user_id.eq(user_id))
            .filter(star_songs::song_id.eq_any(&request.song_ids))
            .execute(&mut database.get().await?)
            .await?;
    }
    if !request.album_ids.is_empty() {
        diesel::delete(star_albums::table)
            .filter(star_albums::user_id.eq(user_id))
            .filter(star_albums::album_id.eq_any(&request.album_ids))
            .execute(&mut database.get().await?)
            .await?;
    }
    if !request.artist_ids.is_empty() {
        diesel::delete(star_artists::table)
            .filter(star_artists::user_id.eq(user_id))
            .filter(star_artists::artist_id.eq_any(&request.artist_ids))
            .execute(&mut database.get().await?)
            .await?;
    }
    Ok(Response)
}
