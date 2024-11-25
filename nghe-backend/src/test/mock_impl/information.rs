use std::borrow::Cow;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use uuid::Uuid;

use super::music_folder;
use crate::file::{self, audio, picture};
use crate::orm::songs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mock<'info, 'path> {
    pub information: audio::Information<'info>,
    pub relative_path: Cow<'path, str>,
}

#[bon::bon]
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

    #[builder(
        builder_type(name = "Builder", vis = "pub"),
        state_mod(name = "builder", vis = "pub"),
        derive(Clone)
    )]
    pub fn builder(
        metadata: Option<audio::Metadata<'static>>,
        song: Option<audio::Song<'static>>,
        album: Option<audio::Album<'static>>,
        artists: Option<audio::Artists<'static>>,
        genres: Option<audio::Genres<'static>>,
        picture: Option<Option<picture::Picture<'static, 'static>>>,
        format: Option<audio::Format>,
        property: Option<audio::Property>,
        relative_path: Option<Cow<'static, str>>,
    ) -> Self {
        let metadata = metadata.unwrap_or_else(|| audio::Metadata {
            song: song.unwrap_or_else(|| Faker.fake()),
            album: album.unwrap_or_else(|| Faker.fake()),
            artists: artists.unwrap_or_else(|| Faker.fake()),
            genres: genres.unwrap_or_else(|| Faker.fake()),
            picture: picture.unwrap_or_else(|| Faker.fake()),
        });
        let file =
            file::Property { format: format.unwrap_or_else(|| Faker.fake()), ..Faker.fake() };
        let property = property.unwrap_or_else(|| audio::Property::default(file.format));
        let relative_path =
            relative_path.map_or_else(|| Faker.fake::<String>().into(), std::convert::Into::into);

        Self { information: audio::Information { metadata, property, file }, relative_path }
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
