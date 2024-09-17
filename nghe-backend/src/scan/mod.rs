use std::borrow::Cow;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use typed_path::Utf8TypedPathBuf;
use uuid::Uuid;

use crate::database::Database;
use crate::filesystem::{self, entry, Entry, Filesystem, Trait};
use crate::orm::music_folders;
use crate::{config, Error};

pub struct Scan<'db, 'fs> {
    pub database: Cow<'db, Database>,
    pub filesystem: filesystem::Impl<'fs>,
    pub id: Uuid,
    pub path: Utf8TypedPathBuf,
}

impl<'db, 'fs> Scan<'db, 'fs> {
    pub async fn new(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        id: Uuid,
    ) -> Result<Self, Error> {
        Self::new_orm(
            database,
            filesystem,
            music_folders::table
                .filter(music_folders::id.eq(id))
                .select(music_folders::MusicFolder::as_select())
                .get_result(&mut database.get().await?)
                .await?,
        )
    }

    pub fn new_orm(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        music_folder: music_folders::MusicFolder,
    ) -> Result<Self, Error> {
        let music_folders::MusicFolder { id, data } = music_folder;
        let music_folders::Data { path, ty } = data;

        let filesystem = filesystem.to_impl(ty.into())?;
        let path = filesystem.path().from_string(path.into_owned());
        Ok(Self { database: Cow::Borrowed(database), filesystem, id, path })
    }

    pub fn start(
        &self,
        config: &config::filesystem::Scan,
    ) -> (JoinHandle<Result<(), Error>>, Receiver<Entry>) {
        let (tx, rx) = tokio::sync::mpsc::channel(config.channel_size);
        let filesystem = self.filesystem.clone().into_owned();
        let sender = entry::Sender { tx, minimum_size: config.minimum_size };
        let prefix = self.path.clone();
        (tokio::spawn(async move { filesystem.scan_folder(sender, prefix.to_path()).await }), rx)
    }
}
