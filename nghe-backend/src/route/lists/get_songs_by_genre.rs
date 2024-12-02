use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::lists::get_songs_by_genre::SongsByGenre;
pub use nghe_api::lists::get_songs_by_genre::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{genres, id3};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    #[check_music_folder]
    {
        Ok(Response {
            songs_by_genre: SongsByGenre {
                song: id3::song::full::query::with_user_id(user_id)
                    .filter(genres::value.eq(request.genre))
                    .limit(request.count.unwrap_or(10).into())
                    .offset(request.offset.unwrap_or(0).into())
                    .get_results(&mut database.get().await?)
                    .await?
                    .into_iter()
                    .map(id3::song::full::Full::try_into)
                    .try_collect()?,
            },
        })
    }
}
