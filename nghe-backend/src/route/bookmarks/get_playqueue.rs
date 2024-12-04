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
pub async fn handler(database: &Database, user_id: Uuid) -> Result<Response, Error> {
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

#[cfg(test)]
mod test {
    use fake::Fake;
    use rstest::rstest;

    use super::*;
    use crate::route::bookmarks::save_playqueue;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        mock.add_music_folder().call().await;

        let mut music_folder_permission = mock.music_folder(0).await;
        let mut music_folder = mock.music_folder(1).await;

        music_folder_permission.add_audio().n_song((2..4).fake()).call().await;
        music_folder.add_audio().n_song((2..4).fake()).call().await;

        let song_ids: Vec<_> = music_folder_permission
            .database
            .keys()
            .copied()
            .chain(music_folder.database.keys().copied())
            .collect();

        let user_id = mock.user_id(0).await;

        save_playqueue::handler(
            mock.database(),
            user_id,
            save_playqueue::Request { ids: song_ids, position: None, current: None },
        )
        .await
        .unwrap();

        let playqueue = handler(mock.database(), user_id).await;
        assert_eq!(playqueue.is_ok(), allow);
    }
}
