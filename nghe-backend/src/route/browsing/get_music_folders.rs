use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
pub use nghe_api::browsing::get_music_folders::{MusicFolder, MusicFolders, Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::orm::{music_folders, permission};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    Ok(Response {
        music_folders: MusicFolders {
            music_folder: music_folders::table
                .filter(permission::with_music_folder_id(user_id))
                .order_by(music_folders::created_at)
                .select((music_folders::id, music_folders::name))
                .get_results::<(Uuid, String)>(&mut database.get().await?)
                .await?
                .into_iter()
                .map(|(id, name)| MusicFolder { id, name })
                .collect(),
        },
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().call().await;
        mock.add_music_folder().allow(allow).call().await;

        let user_id = mock.user_id(0).await;
        let music_folders = handler(mock.database(), user_id, Request)
            .await
            .unwrap()
            .music_folders
            .music_folder
            .into_iter()
            .map(|music_folder| music_folder.id)
            .collect::<Vec<_>>();

        if allow {
            assert_eq!(
                music_folders,
                &[mock.music_folder(0).await.id(), mock.music_folder(1).await.id()]
            );
        } else {
            assert_eq!(music_folders, &[mock.music_folder(0).await.id()]);
        }
    }
}
