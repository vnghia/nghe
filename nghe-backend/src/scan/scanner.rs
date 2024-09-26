use std::borrow::Cow;
use std::sync::Arc;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lofty::config::ParseOptions;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use typed_path::Utf8TypedPathBuf;
use uuid::Uuid;

use crate::database::Database;
use crate::file::File;
use crate::filesystem::{self, entry, Entry, Filesystem, Trait};
use crate::orm::music_folders;
use crate::{config, Error};

#[derive(Debug, Clone)]
pub struct Config {
    pub lofty: ParseOptions,
    pub scan: config::filesystem::Scan,
    pub parsing: config::Parsing,
}

#[derive(Debug, Clone)]
pub struct Scanner<'db, 'fs> {
    pub database: Cow<'db, Database>,
    pub filesystem: filesystem::Impl<'fs>,
    pub config: Config,
    pub id: Uuid,
    pub path: Utf8TypedPathBuf,
}

impl<'db, 'fs> Scanner<'db, 'fs> {
    pub async fn new(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        config: Config,
        id: Uuid,
    ) -> Result<Self, Error> {
        Self::new_orm(
            database,
            filesystem,
            config,
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
        config: Config,
        music_folder: music_folders::MusicFolder,
    ) -> Result<Self, Error> {
        let music_folders::MusicFolder { id, data } = music_folder;
        let music_folders::Data { path, ty } = data;

        let filesystem = filesystem.to_impl(ty.into())?;
        let path = filesystem.path().from_string(path.into_owned());
        Ok(Self { database: Cow::Borrowed(database), filesystem, config, id, path })
    }

    fn into_owned(self) -> Scanner<'static, 'static> {
        Scanner {
            database: Cow::Owned(self.database.into_owned()),
            filesystem: self.filesystem.into_owned(),
            ..self
        }
    }

    fn init(&self) -> (JoinHandle<Result<(), Error>>, Arc<Semaphore>, Receiver<Entry>) {
        let config = self.config.scan;
        let (tx, rx) = tokio::sync::mpsc::channel(config.channel_size);
        let filesystem = self.filesystem.clone().into_owned();
        let sender = entry::Sender { tx, minimum_size: config.minimum_size };
        let prefix = self.path.clone();
        (
            tokio::spawn(async move { filesystem.scan_folder(sender, prefix.to_path()).await }),
            Arc::new(Semaphore::const_new(config.pool_size)),
            rx,
        )
    }

    async fn one(&self, entry: &Entry) -> Result<(), Error> {
        let audio = File::new(self.filesystem.read(entry.path.to_path()).await?, entry.format)?
            .audio(self.config.lofty)?;
        let information = audio.extract(&self.config.parsing)?;

        let album_id = information.upsert_album(&self.database, self.id).await?;

        information
            .upsert(&self.database, album_id, entry.relative_path(&self.path)?.as_str(), None)
            .await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Error> {
        let (scan_handle, permit, mut rx) = self.init();
        let mut join_set = tokio::task::JoinSet::new();

        while let Some(entry) = rx.recv().await {
            let permit = permit.clone().acquire_owned().await?;
            let scan = self.clone().into_owned();
            join_set.spawn(async move {
                let _guard = permit;
                scan.one(&entry).await
            });
        }

        while let Some(result) = join_set.join_next().await {
            result??;
        }
        scan_handle.await??;

        Ok(())
    }
}
