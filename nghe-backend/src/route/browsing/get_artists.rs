use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_api::browsing::get_artists::Artists;
pub use nghe_api::browsing::get_artists::{Index, Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::config;
use crate::database::Database;
use crate::error::Error;
use crate::orm::id3;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let ignored_articles = database.get_config::<config::Index>().await?;

    let artists = if let Some(music_folder_ids) = request.music_folder_ids {
        id3::artist::query::music_folder(user_id, &music_folder_ids)
            .get_results(&mut database.get().await?)
            .await?
    } else {
        id3::artist::query::permission(user_id).get_results(&mut database.get().await?).await?
    };

    let index = artists
        .into_iter()
        .into_group_map_by(|artist| artist.index.clone())
        .into_iter()
        .map(|(name, artist)| {
            Ok::<_, Error>(Index {
                name,
                artist: artist.into_iter().map(id3::artist::Artist::try_into_api).try_collect()?,
            })
        })
        .try_collect()?;

    Ok(Response { artists: Artists { ignored_articles, index } })
}
