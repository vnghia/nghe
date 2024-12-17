use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::browsing::get_artist_info2::ArtistInfo2;
pub use nghe_api::browsing::get_artist_info2::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{artists, id3};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let music_brainz_id = id3::artist::query::with_user_id(user_id)
        .filter(artists::id.eq(request.id))
        .select(artists::mbz_id)
        .get_result(&mut database.get().await?)
        .await?;
    Ok(Response { artist_info2: ArtistInfo2 { music_brainz_id } })
}
