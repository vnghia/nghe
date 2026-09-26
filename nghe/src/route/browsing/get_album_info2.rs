use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_api::browsing::get_album_info2::AlbumInfo;
pub use nghe_api::browsing::get_album_info2::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{albums, permission};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let music_brainz_id = albums::table
        .filter(permission::with_album(user_id))
        .filter(albums::id.eq(request.id))
        .select(albums::mbz_id)
        .get_result(&mut database.get().await?)
        .await?;
    Ok(Response { album_info: AlbumInfo { music_brainz_id } })
}
