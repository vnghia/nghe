use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::lists::get_random_songs::RandomSong;
pub use nghe_api::lists::get_random_songs::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{albums, function, genres, id3};
use crate::schema::songs;
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    #[check_music_folder]
    {
        let mut query = id3::song::with_album_genres::query::with_user_id(user_id)
            .limit(request.size.unwrap_or(10).into())
            .order_by(function::random())
            .into_boxed();

        if let Some(genre) = request.genre {
            query = query.filter(genres::value.eq(genre));
        }
        if let Some(from_year) = request.from_year {
            let from_year: i16 = from_year.try_into()?;
            query = query.filter(songs::year.ge(from_year).or(albums::year.ge(from_year)));
        }
        if let Some(to_year) = request.to_year {
            let to_year: i16 = to_year.try_into()?;
            query = query.filter(songs::year.le(to_year).or(albums::year.le(to_year)));
        }

        Ok(Response {
            random_songs: RandomSong {
                song: query
                    .get_results(&mut database.get().await?)
                    .await?
                    .into_iter()
                    .map(id3::song::with_album_genres::WithAlbumGenres::try_into)
                    .try_collect()?,
            },
        })
    }
}
