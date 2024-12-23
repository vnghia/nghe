use std::borrow::Cow;
use std::io::{Cursor, Write};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use uuid::Uuid;

use super::music_folder;
use crate::file::{self, audio, picture};
use crate::orm::{albums, songs};
use crate::test::assets;
use crate::test::file::audio::dump::Metadata as _;
use crate::test::filesystem::Trait as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mock<'info, 'path> {
    pub information: audio::Information<'info>,
    pub dir_picture: Option<picture::Picture<'static>>,
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
        let album_id = upsert.foreign.album_id;
        let album = audio::Album::query_upsert(mock, upsert.foreign.album_id).await;
        let artists = audio::Artists::query(mock, id).await;
        let genres = audio::Genres::query(mock, id).await;
        let picture = picture::Picture::query_song(mock, id).await;

        let dir_picture = picture::Picture::query_album(mock, album_id).await;

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
                file: upsert.data.file.try_into().unwrap(),
            },
            dir_picture,
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
        picture: Option<Option<picture::Picture<'static>>>,
        format: Option<audio::Format>,
        file_property: Option<file::Property<audio::Format>>,
        property: Option<audio::Property>,
        dir_picture: Option<Option<picture::Picture<'static>>>,
        relative_path: Option<Cow<'static, str>>,
    ) -> Self {
        let metadata = metadata.unwrap_or_else(|| audio::Metadata {
            song: song.unwrap_or_else(|| Faker.fake()),
            album: album.unwrap_or_else(|| Faker.fake()),
            artists: artists.unwrap_or_else(|| Faker.fake()),
            genres: genres.unwrap_or_else(|| Faker.fake()),
            picture: picture.unwrap_or_else(|| Faker.fake()),
        });
        let file = file_property.unwrap_or_else(|| file::Property {
            format: format.unwrap_or_else(|| Faker.fake()),
            ..Faker.fake()
        });
        let property = property.unwrap_or_else(|| audio::Property::default(file.format));

        let dir_picture = dir_picture.unwrap_or_else(|| Faker.fake());
        let relative_path =
            relative_path.map_or_else(|| Faker.fake::<String>().into(), std::convert::Into::into);

        Self {
            information: audio::Information { metadata, property, file },
            dir_picture,
            relative_path,
        }
    }
}

impl Mock<'_, '_> {
    pub async fn upsert(
        &self,
        music_folder: &music_folder::Mock<'_>,
        song_id: impl Into<Option<Uuid>>,
    ) -> Uuid {
        let database = music_folder.database();
        let dir_picture_id = if let Some(ref dir) = music_folder.config.cover_art.dir
            && let Some(ref picture) = self.dir_picture
        {
            Some(picture.upsert(database, dir, None::<&str>).await.unwrap())
        } else {
            None
        };

        self.information
            .upsert(
                database,
                &music_folder.config,
                albums::Foreign {
                    music_folder_id: music_folder.id(),
                    cover_art_id: dir_picture_id,
                },
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

    pub async fn dump(self, music_folder: &music_folder::Mock<'_>) -> Self {
        let path = music_folder.path().join(&self.relative_path);
        let path = path.to_path();

        let format = self.information.file.format;
        let data = tokio::fs::read(assets::path(format).as_str()).await.unwrap();
        let mut asset = Cursor::new(data.clone());
        let mut file =
            file::File::new(format, data).unwrap().audio(music_folder.config.lofty).unwrap();
        asset.set_position(0);

        file.clear()
            .dump_metadata(&music_folder.config.parsing, self.information.metadata.clone())
            .save_to(&mut asset, music_folder.write_options());

        asset.flush().unwrap();
        asset.set_position(0);
        let data = asset.into_inner();

        let filesystem = &music_folder.to_impl();
        filesystem.write(path, &data).await;

        let cover_art_config = &music_folder.config.cover_art;
        let parent = path.parent().unwrap();
        let dir_picture = if let Some(picture) =
            picture::Picture::scan_filesystem(filesystem, cover_art_config, parent).await
        {
            Some(picture)
        } else if let Some(picture) = self.dir_picture {
            let path = parent.join(picture.property.format.name());
            filesystem.write(path.to_path(), &picture.data).await;
            Some(picture)
        } else {
            None
        };

        Self {
            information: audio::Information {
                file: file::Property::new(format, &data).unwrap(),
                ..self.information
            },
            dir_picture,
            ..self
        }
    }

    pub fn with_dir_picture(self, dir_picture: Option<picture::Picture<'static>>) -> Self {
        Self { dir_picture, ..self }
    }
}

impl<'path> Mock<'_, 'path> {
    pub fn with_relative_path(self, relative_path: Cow<'path, str>) -> Self {
        Self { relative_path, ..self }
    }
}
