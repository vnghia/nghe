use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_api::browsing::get_artists::Artists;
pub use nghe_api::browsing::get_artists::{Index, Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::id3;
use crate::{Error, config};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let ignored_articles = database.get_config::<config::Index>().await?;

    let artists =
        #[check_music_folder]
        id3::artist::query::with_user_id(user_id).get_results(&mut database.get().await?).await?;

    let index = artists
        .into_iter()
        .into_group_map_by(|artist| artist.index.clone())
        .into_iter()
        .sorted_by(|lhs, rhs| Ord::cmp(&lhs.0, &rhs.0))
        .map(|(name, artist)| {
            Ok::<_, Error>(Index {
                name,
                artist: artist
                    .into_iter()
                    .sorted_by(|lhs, rhs| Ord::cmp(&lhs.required.name, &rhs.required.name))
                    .map(id3::artist::Artist::try_into)
                    .try_collect()?,
            })
        })
        .try_collect()?;

    Ok(Response { artists: Artists { ignored_articles, index } })
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use concat_string::concat_string;
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_sorted(#[future(awt)] mock: Mock) {
        mock.add_audio_artist(
            0,
            ["A1".into(), "A2".into(), "C1".into(), "C2".into()],
            ["B1".into(), "B2".into()],
            false,
            1,
        )
        .await;

        let index =
            handler(mock.database(), mock.user_id(0).await, Request { music_folder_ids: None })
                .await
                .unwrap()
                .artists
                .index;

        for (i, index) in index.into_iter().enumerate() {
            let name =
                char::from_u32((b'A' + u8::try_from(i).unwrap()).into()).unwrap().to_string();
            assert_eq!(index.name, name);
            for (j, artist) in index.artist.into_iter().enumerate() {
                let name = concat_string!(&name, (j + 1).to_string());
                assert_eq!(artist.required.name, name);
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_check_music_folder(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        mock.add_music_folder().allow(false).call().await;
        mock.add_music_folder().call().await;

        let mut music_folder_deny = mock.music_folder(0).await;
        let mut music_folder_allow = mock.music_folder(1).await;
        music_folder_deny.add_audio().n_song(10).call().await;
        music_folder_allow.add_audio().n_song(10).call().await;

        let user_id = mock.user_id(0).await;
        let with_user_id =
            handler(mock.database(), user_id, Request { music_folder_ids: None }).await.unwrap();
        let with_music_folder = handler(
            mock.database(),
            user_id,
            Request {
                music_folder_ids: Some(vec![music_folder_deny.id(), music_folder_allow.id()]),
            },
        )
        .await
        .unwrap();
        assert_eq!(with_user_id, with_music_folder);
    }
}
