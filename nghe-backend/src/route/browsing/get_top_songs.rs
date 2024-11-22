use diesel::dsl::sum;
use diesel::{ExpressionMethods, JoinOnDsl, PgSortExpressionMethods as _, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::browsing::get_top_songs::TopSongs;
pub use nghe_api::browsing::get_top_songs::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{artists, id3, playbacks, songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    // Since each playback count is accounted for a user, we make a sum here to get the total
    // playback count.
    Ok(Response {
        top_songs: TopSongs {
            song: id3::song::full::query::with_user_id(user_id)
                .filter(artists::name.eq(request.artist))
                .left_join(playbacks::table.on(playbacks::song_id.eq(songs::id)))
                .order_by(sum(playbacks::count).desc().nulls_last())
                .limit(request.count.unwrap_or(50).into())
                .get_results(&mut database.get().await?)
                .await?
                .into_iter()
                .map(id3::song::full::Full::try_into)
                .try_collect()?,
        },
    })
}
