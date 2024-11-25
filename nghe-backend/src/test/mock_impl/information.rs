use std::borrow::Cow;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Dummy, Fake, Faker};
use uuid::Uuid;

use super::music_folder;
use crate::file::{audio, picture};
use crate::orm::songs;

#[derive(Debug, Clone, Dummy, PartialEq, Eq)]
pub struct Mock<'info, 'path> {
    pub information: audio::Information<'info>,
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub relative_path: Cow<'path, str>,
}

impl Mock<'static, 'static> {
    pub async fn query_upsert(mock: &super::Mock, id: Uuid) -> songs::Upsert<'static> {
        songs::table
            .filter(songs::id.eq(id))
            .select(songs::Upsert::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap()
    }

    pub async fn query_data(mock: &super::Mock, id: Uuid) -> songs::Data<'static> {
        Self::query_upsert(mock, id).await.data
    }

    pub async fn query(mock: &super::Mock, id: Uuid) -> Self {
        let upsert = Self::query_upsert(mock, id).await;
        let album = audio::Album::query_upsert(mock, upsert.foreign.album_id).await;
        let artists = audio::Artists::query(mock, id).await;
        let genres = audio::Genres::query(mock, id).await;
        let picture = picture::Picture::query_song(mock, id).await;

        Self {
            information: audio::Information {
                metadata: audio::Metadata {
                    song: upsert.data.song.try_into().unwrap(),
                    album: album.data.try_into().unwrap(),
                    artists,
                    genres,
                    picture,
                },
                property: upsert.data.property.try_into().unwrap(),
                file: upsert.data.file.into(),
            },
            relative_path: upsert.relative_path,
        }
    }
}

impl Mock<'_, '_> {
    pub async fn upsert(
        &self,
        music_folder: &music_folder::Mock<'_>,
        song_id: impl Into<Option<Uuid>>,
    ) -> Uuid {
        self.information
            .upsert(
                music_folder.database(),
                &music_folder.config,
                music_folder.id().into(),
                self.relative_path.as_str(),
                song_id,
            )
            .await
            .unwrap()
    }

    pub async fn upsert_mock(
        &self,
        mock: &super::Mock,
        index: usize,
        song_id: impl Into<Option<Uuid>>,
    ) -> Uuid {
        let music_folder = mock.music_folder(index).await;
        self.upsert(&music_folder, song_id).await
    }
}
