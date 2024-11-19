use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_artist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::orm::{artists, id3};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        artist: id3::artist_with_albums::query::with_user_id(user_id)
            .filter(artists::id.eq(request.id))
            .get_result(&mut database.get().await?)
            .await?
            .try_into_api(database)
            .await?,
    })
}
