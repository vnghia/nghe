use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_lite::{stream, StreamExt as _};
use nghe_api::bookmarks::get_playqueue::Playqueue;
pub use nghe_api::bookmarks::get_playqueue::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{id3, playqueues, songs};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(
        if let Some(data) = playqueues::table
            .filter(playqueues::user_id.eq(user_id))
            .select(playqueues::Data::as_select())
            .get_result(&mut database.get().await?)
            .await
            .optional()?
        {
            let entry = stream::iter(data.ids)
                .then(async |id| {
                    id3::song::short::query::with_user_id(user_id)
                        .filter(songs::id.eq(id))
                        .get_result(&mut database.get().await?)
                        .await?
                        .try_into()
                })
                .try_collect()
                .await?;

            Response {
                playqueue: Playqueue {
                    entry,
                    current: data.current,
                    position: data.position.map(i64::try_into).transpose()?,
                },
            }
        } else {
            Response::default()
        },
    )
}
