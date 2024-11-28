use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use nghe_api::playlists::create_playlist::CreateOrUpdate;
pub use nghe_api::playlists::create_playlist::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use super::get_playlist;
use crate::database::Database;
use crate::orm::upsert::Insert;
use crate::orm::{playlist, playlists, playlists_songs, playlists_users};
use crate::Error;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let playlist_id = match request.create_or_update {
        CreateOrUpdate::Create { name } => {
            let playlist_id = playlists::Upsert { name: Some(name.into()), ..Default::default() }
                .insert(database)
                .await?;
            playlists_users::Upsert::insert_owner(database, playlist_id, user_id).await?;
            playlist_id
        }
        CreateOrUpdate::Update { playlist_id } => {
            playlist::permission::check_write(database, playlist_id, user_id, false).await?;
            diesel::delete(playlists_songs::table)
                .filter(playlists_songs::playlist_id.eq(playlist_id))
                .execute(&mut database.get().await?)
                .await?;
            playlist_id
        }
    };
    if let Some(ref song_ids) = request.song_ids {
        playlists_songs::Upsert::upserts(database, playlist_id, song_ids).await?;
    }

    Ok(Response {
        playlist: get_playlist::handler(
            database,
            user_id,
            get_playlist::Request { id: playlist_id },
        )
        .await?
        .playlist,
    })
}
