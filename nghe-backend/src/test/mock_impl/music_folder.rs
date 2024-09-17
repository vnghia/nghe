use std::io::{Cursor, Write};

use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::file::audio;
use crate::filesystem::Trait as _;
use crate::orm::music_folders;
use crate::test::assets;
use crate::test::filesystem::{self, Trait as _};
use crate::test::media::dump::Metadata as _;

pub struct Mock<'a> {
    mock: &'a super::Mock,
    pub music_folder: music_folders::MusicFolder<'static>,
}

#[bon::bon]
impl<'a> Mock<'a> {
    pub async fn new(mock: &'a super::Mock, index: usize) -> Self {
        Self {
            mock,
            music_folder: music_folders::table
                .select(music_folders::MusicFolder::as_select())
                .order_by(music_folders::created_at)
                .offset(index.try_into().unwrap())
                .first(&mut mock.get().await)
                .await
                .unwrap(),
        }
    }

    pub fn absolutize(&self, path: impl AsRef<str>) -> Utf8TypedPathBuf {
        self.to_impl().path().from_str(&self.music_folder.data.path).join(path)
    }

    pub fn to_impl(&self) -> filesystem::Impl<'_> {
        self.mock.to_impl(self.music_folder.data.ty.into())
    }

    #[builder]
    pub async fn add_audio(
        &self,
        path: Option<Utf8TypedPath<'_>>,
        #[builder(default = (0..3).fake::<usize>())] depth: usize,
        #[builder(default = Faker.fake::<audio::Format>())] format: audio::Format,
        #[builder(default = Faker.fake::<audio::Audio>())] audio: audio::Audio<'_>,
    ) -> &Self {
        let path = if let Some(path) = path {
            self.absolutize(path)
        } else {
            self.absolutize(self.to_impl().fake_path(depth))
        }
        .with_extension(format.as_ref());

        let data = tokio::fs::read(assets::path(format).as_str()).await.unwrap();
        let mut asset = Cursor::new(data.clone());
        let mut file = audio::File::read_from(data, self.mock.config.lofty_parse, format).unwrap();
        asset.set_position(0);

        file.clear()
            .dump_metadata(&self.mock.config.parsing, audio.metadata)
            .save_to(&mut asset, self.mock.config.lofty_write);

        asset.flush().unwrap();
        asset.set_position(0);
        self.to_impl().write(path.to_path(), &asset.into_inner()).await;

        self
    }

    pub async fn file(&self, path: Utf8TypedPath<'_>, format: audio::Format) -> audio::File {
        let path = self.absolutize(path).with_extension(format.as_ref());
        audio::File::read_from(
            self.to_impl().read(path.to_path()).await.unwrap(),
            self.mock.config.lofty_parse,
            format,
        )
        .unwrap()
    }
}
