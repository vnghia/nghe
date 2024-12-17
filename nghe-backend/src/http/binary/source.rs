use diesel_async::RunQueryDsl;
use typed_path::Utf8TypedPathBuf;
use uuid::Uuid;

use super::property;
use crate::Error;
use crate::database::Database;
use crate::file::{self, audio};
use crate::filesystem::{self, Filesystem};
use crate::orm::binary;

pub struct Source<P: property::Trait> {
    pub path: Utf8TypedPathBuf,
    pub property: P,
}

impl Source<file::Property<audio::Format>> {
    pub async fn audio<'fs>(
        database: &Database,
        filesystem: &'fs Filesystem,
        user_id: Uuid,
        song_id: Uuid,
    ) -> Result<(filesystem::Impl<'fs>, Self), Error> {
        let audio = binary::source::audio::query(user_id, song_id)
            .get_result(&mut database.get().await?)
            .await?;
        let filesystem = filesystem.to_impl(audio.music_folder.ty.into())?;
        let path = filesystem
            .path()
            .from_string(audio.music_folder.path.into_owned())
            .join(audio.relative_path);
        Ok((filesystem, Self { path, property: audio.property.try_into()? }))
    }
}
