use axum::extract::State;
use itertools::Itertools;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use nghe_types::open_subsonic::common::id::{MediaType, MediaTypedId};

use super::get_artists::get_artists;
use crate::Database;

add_common_validate!(GetIndexesParams);
add_axum_response!(GetIndexesBody);

pub async fn get_indexed_handler(
    State(database): State<Database>,
    req: GetIndexesRequest,
) -> GetIndexesJsonResponse {
    let indexed_artists =
        get_artists(&database.pool, req.user_id, &req.params.music_folder_ids).await?;
    let index = indexed_artists
        .index
        .into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
        .map(|i| Index {
            name: i.name,
            children: i
                .artists
                .into_iter()
                .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
                .map(|c| ChildItem {
                    id: MediaTypedId { t: Some(MediaType::Aritst), id: c.id },
                    parent: None,
                    is_dir: None,
                    name: Some(c.name),
                    title: None,
                    cover_art: None,
                })
                .collect(),
        })
        .collect();
    Ok(axum::Json(
        GetIndexesBody {
            indexes: Indexes { ignored_articles: indexed_artists.ignored_articles, index },
        }
        .into(),
    ))
}
