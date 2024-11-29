use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::lists::get_starred2::Starred2;
pub use nghe_api::lists::get_starred2::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{id3, star_albums, star_artists, star_songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    #[check_music_folder]
    {
        let artist = id3::artist::query::with_user_id(user_id)
            .filter(star_artists::user_id.eq(user_id))
            .get_results(&mut database.get().await?)
            .await?;
        let album = id3::album::short::query::with_user_id(user_id)
            .filter(star_albums::user_id.eq(user_id))
            .get_results(&mut database.get().await?)
            .await?;
        let song = id3::song::short::query::with_user_id(user_id)
            .filter(star_songs::user_id.eq(user_id))
            .get_results(&mut database.get().await?)
            .await?;

        Ok(Response {
            starred2: Starred2 {
                artist: artist.into_iter().map(id3::artist::Artist::try_into).try_collect()?,
                album: album.into_iter().map(id3::album::short::Short::try_into).try_collect()?,
                song: song.into_iter().map(id3::song::short::Short::try_into).try_collect()?,
            },
        })
    }
}
