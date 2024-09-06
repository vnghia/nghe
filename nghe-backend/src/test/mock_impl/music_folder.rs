use std::io::{Cursor, Write};

use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::filesystem::Trait;
use crate::media::file;
use crate::orm::music_folders;
use crate::test::assets;
use crate::test::filesystem::{self, MockTrait};
use crate::test::media::MetadataDumper;

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
                .order_by(music_folders::schema::created_at)
                .offset(index.try_into().unwrap())
                .first(&mut mock.get().await)
                .await
                .unwrap(),
        }
    }

    pub fn absolutize(&self, path: impl Into<Option<Utf8TypedPath<'_>>>) -> Utf8TypedPathBuf {
        let music_folder_path = Utf8TypedPath::from(self.music_folder.data.path.as_ref());
        if let Some(path) = path.into() {
            music_folder_path.join(path)
        } else {
            music_folder_path.join(Faker.fake::<String>())
        }
    }

    pub fn to_impl(&self) -> filesystem::MockImpl<'_> {
        self.mock.to_impl(self.music_folder.data.filesystem_type.into())
    }

    #[builder]
    pub async fn add_media(
        &self,
        path: Option<Utf8TypedPath<'_>>,
        #[builder(default = Faker.fake::<file::Type>())] file_type: file::Type,
        #[builder(default = Faker.fake::<file::Media>())] media: file::Media<'_>,
    ) -> &Self {
        let path = self.absolutize(path).with_extension(assets::ext(file_type));

        let mut asset =
            Cursor::new(tokio::fs::read(assets::path(file_type).as_str()).await.unwrap());
        let mut file =
            file::File::read_from(&mut asset, self.mock.lofty_parse_options, file_type).unwrap();
        file.clear().dump_metadata(&self.mock.parsing_config, media.metadata);
        asset.set_position(0);
        file.save_to(&mut asset, self.mock.lofty_write_options);

        asset.flush().unwrap();
        asset.set_position(0);
        self.to_impl().write(path.to_path(), &asset.into_inner()).await;

        self
    }

    pub async fn file(&self, path: Utf8TypedPath<'_>, file_type: file::Type) -> file::File {
        let path = self.absolutize(path).with_extension(assets::ext(file_type));
        let mut data = Cursor::new(self.to_impl().read(path.to_path()).await.unwrap());
        file::File::read_from(&mut data, self.mock.lofty_parse_options, file_type).unwrap()
    }
}
