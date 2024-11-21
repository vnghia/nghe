use diesel::dsl::{max, sum};
use diesel::{ExpressionMethods, JoinOnDsl, PgSortExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::lists::get_album_list2::{AlbumList2, ByYear, Type};
pub use nghe_api::lists::get_album_list2::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{albums, function, genres, id3, playbacks, songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    #[check_music_folder]
    {
        let query = id3::album::with_durations::query::with_user_id(user_id)
            .limit(request.size.unwrap_or(10).into())
            .offset(request.offset.unwrap_or(0).into());

        let albums = match request.ty {
            Type::Random => {
                query.order_by(function::random()).get_results(&mut database.get().await?).await?
            }
            Type::Newest => {
                query
                    .order_by(albums::created_at.desc())
                    .get_results(&mut database.get().await?)
                    .await?
            }
            Type::Frequent => {
                // Since each playback count is accounted for a song, we make a sum here to get the
                // playback count of the whole album.
                query
                    .inner_join(playbacks::table.on(playbacks::song_id.eq(songs::id)))
                    .filter(playbacks::user_id.eq(user_id))
                    .order_by(sum(playbacks::count).desc().nulls_last())
                    .get_results(&mut database.get().await?)
                    .await?
            }
            Type::Recent => {
                query
                    .inner_join(playbacks::table.on(playbacks::song_id.eq(songs::id)))
                    .filter(playbacks::user_id.eq(user_id))
                    .order_by(max(playbacks::updated_at).desc().nulls_last())
                    .get_results(&mut database.get().await?)
                    .await?
            }
            Type::AlphabeticalByName => query.get_results(&mut database.get().await?).await?,
            Type::ByYear(ByYear { from_year, to_year }) => {
                let from_year: i16 = from_year.try_into()?;
                let to_year: i16 = to_year.try_into()?;
                if from_year < to_year {
                    query
                        .filter(albums::year.ge(from_year))
                        .filter(albums::year.le(to_year))
                        .order_by(albums::year.asc())
                        .get_results(&mut database.get().await?)
                        .await?
                } else {
                    query
                        .filter(albums::year.ge(to_year))
                        .filter(albums::year.le(from_year))
                        .order_by(albums::year.desc())
                        .get_results(&mut database.get().await?)
                        .await?
                }
            }
            Type::ByGenre { genre } => {
                query
                    .filter(genres::value.eq(genre))
                    .get_results(&mut database.get().await?)
                    .await?
            }
        };

        Ok(Response {
            album_list2: AlbumList2 {
                album: albums
                    .into_iter()
                    .map(id3::album::with_durations::WithDurations::try_into)
                    .try_collect()?,
            },
        })
    }
}
