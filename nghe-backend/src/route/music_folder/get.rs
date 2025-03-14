use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
pub use nghe_api::music_folder::get::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::orm::{music_folders, user_music_folder_permissions};

#[handler(internal = true)]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let music_folder_id = request.id;
    user_music_folder_permissions::Permission::check_owner(database, user_id, music_folder_id)
        .await?;
    Ok(music_folders::stat::query::unchecked()
        .filter(music_folders::id.eq(music_folder_id))
        .get_result(&mut database.get().await?)
        .await?
        .into())
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::num::NonZero;

    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::{self, audio};
    use crate::orm::users;
    use crate::test::{Mock, mock};

    #[rstest]
    #[case(0, &[])]
    #[case(2, &[10, 20])]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(0, 1)]
        mock: Mock,
        #[case] n_album: u64,
        #[case] n_song: &[u64],
    ) {
        let user_id =
            mock.add_user().role(users::Role { admin: true }).call().await.user_id(0).await;
        let mut music_folder = mock.music_folder(0).await;

        for i in 0..n_album {
            let album: audio::Album = Faker.fake();
            for _ in 0..n_song[usize::try_from(i).unwrap()] {
                music_folder
                    .add_audio()
                    .album(album.clone())
                    .file_property(file::Property {
                        size: NonZero::new((100..1000).fake()).unwrap(),
                        ..Faker.fake()
                    })
                    .call()
                    .await;
            }
        }

        let total_size = music_folder
            .database
            .iter()
            .fold(0u64, |size, song| size + u64::from(song.1.information.file.size.get()));

        let response =
            handler(mock.database(), user_id, Request { id: music_folder.id() }).await.unwrap();
        assert_eq!(response.album_count, n_album);
        assert_eq!(response.song_count, n_song.iter().sum::<u64>());
        assert_eq!(response.total_size, total_size);
    }
}
