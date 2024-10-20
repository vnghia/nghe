use diesel_async::RunQueryDsl;
use typed_path::Utf8TypedPathBuf;
use uuid::Uuid;

use crate::database::Database;
use crate::file::{self, audio};
use crate::filesystem::Filesystem;
use crate::orm::retriever;
use crate::response::Binary;
use crate::{filesystem, Error};

pub struct Song<'fs> {
    pub filesystem: filesystem::Impl<'fs>,
    pub path: Utf8TypedPathBuf,
    pub property: file::Property<audio::Format>,
}

impl<'fs> Song<'fs> {
    pub async fn new(
        database: &Database,
        filesystem: &'fs Filesystem,
        user_id: Uuid,
        song_id: Uuid,
    ) -> Result<Self, Error> {
        let song =
            retriever::query(user_id, song_id).get_result(&mut database.get().await?).await?;
        let filesystem = filesystem.to_impl(song.music_folder.ty.into())?;
        let path = filesystem
            .path()
            .from_string(song.music_folder.path.into_owned())
            .join(song.relative_path);
        let property = song.property.into();
        Ok(Self { filesystem, path, property })
    }
}
