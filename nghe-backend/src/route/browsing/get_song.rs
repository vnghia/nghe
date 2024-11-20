use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_song::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::orm::{id3, songs};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        song: id3::song::with_album_genres::query::with_user_id(user_id)
            .filter(songs::id.eq(request.id))
            .get_result(&mut database.get().await?)
            .await?
            .try_into()?,
    })
}
