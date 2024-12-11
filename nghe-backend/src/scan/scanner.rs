use std::borrow::Cow;
use std::sync::Arc;

use diesel::{ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lofty::config::ParseOptions;
use loole::Receiver;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tracing::{instrument, Instrument};
use typed_path::Utf8TypedPath;
use uuid::Uuid;

use crate::database::Database;
use crate::file::{self, audio, picture, File};
use crate::filesystem::{self, entry, Entry, Filesystem, Trait};
use crate::integration::Informant;
use crate::orm::{albums, music_folders, songs};
use crate::{config, error, Error};

#[derive(Debug, Clone)]
pub struct Config {
    pub lofty: ParseOptions,
    pub scan: config::filesystem::Scan,
    pub parsing: config::Parsing,
    pub index: config::Index,
    pub cover_art: config::CoverArt,
}

#[derive(Clone)]
pub struct Scanner<'db, 'fs, 'mf> {
    pub database: Cow<'db, Database>,
    pub filesystem: filesystem::Impl<'fs>,
    pub config: Config,
    pub informant: Informant,
    pub music_folder: music_folders::MusicFolder<'mf>,
}

impl<'db, 'fs, 'mf> Scanner<'db, 'fs, 'mf> {
    pub async fn new(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        config: Config,
        informant: Informant,
        music_folder_id: Uuid,
    ) -> Result<Self, Error> {
        Self::new_orm(
            database,
            filesystem,
            config,
            informant,
            music_folders::MusicFolder::query(database, music_folder_id).await?,
        )
    }

    pub fn new_orm(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        config: Config,
        informant: Informant,
        music_folder: music_folders::MusicFolder<'mf>,
    ) -> Result<Self, Error> {
        let filesystem = filesystem.to_impl(music_folder.data.ty.into())?;
        Ok(Self { database: Cow::Borrowed(database), filesystem, config, informant, music_folder })
    }

    pub fn into_owned(self) -> Scanner<'static, 'static, 'static> {
        Scanner {
            database: Cow::Owned(self.database.into_owned()),
            filesystem: self.filesystem.into_owned(),
            music_folder: self.music_folder.into_owned(),
            ..self
        }
    }

    fn path(&self) -> Utf8TypedPath {
        self.filesystem.path().from_str(&self.music_folder.data.path)
    }

    fn relative_path<'entry>(&self, entry: &'entry Entry) -> Result<Utf8TypedPath<'entry>, Error> {
        entry.path.strip_prefix(&self.music_folder.data.path).map_err(Error::from)
    }

    fn init(&self) -> (JoinHandle<Result<(), Error>>, Arc<Semaphore>, Receiver<Entry>) {
        let config = self.config.scan;
        let (tx, rx) = crate::sync::channel(config.channel_size);
        let filesystem = self.filesystem.clone().into_owned();
        let sender = entry::Sender { tx, minimum_size: config.minimum_size };
        let prefix = self.path().to_path_buf();
        (
            tokio::spawn(async move { filesystem.scan_folder(sender, prefix.to_path()).await }),
            Arc::new(Semaphore::const_new(config.pool_size)),
            rx,
        )
    }

    async fn set_scanned_at(
        &self,
        entry: &Entry,
    ) -> Result<Option<(Uuid, time::OffsetDateTime)>, Error> {
        let song_path = diesel::alias!(songs as song_path);
        diesel::update(songs::table)
            .filter(
                songs::id.nullable().eq(song_path
                    .inner_join(albums::table)
                    .filter(albums::music_folder_id.eq(self.music_folder.id))
                    .filter(
                        song_path
                            .field(songs::relative_path)
                            .eq(entry.relative_path(&self.music_folder.data.path)?.as_str()),
                    )
                    .select(song_path.field(songs::id))
                    .single_value()),
            )
            .set(songs::scanned_at.eq(time::OffsetDateTime::now_utc()))
            .returning((songs::id, songs::updated_at))
            .get_result(&mut self.database.get().await?)
            .await
            .optional()
            .map_err(Error::from)
    }

    async fn query_hash_size(
        &self,
        property: &file::Property<audio::Format>,
    ) -> Result<Option<(Uuid, String)>, Error> {
        songs::table
            .inner_join(albums::table)
            .filter(albums::music_folder_id.eq(self.music_folder.id))
            .filter(songs::file_hash.eq(property.hash.cast_signed()))
            .filter(songs::file_size.eq(property.size.get().cast_signed()))
            .select((songs::id, songs::relative_path))
            .get_result(&mut self.database.get().await?)
            .await
            .optional()
            .map_err(Error::from)
    }

    #[instrument(
        skip_all,
        fields(path = %entry.path, last_modified = ?entry.last_modified),
        ret(level = "debug"),
        err(Debug)
    )]
    async fn one(&self, entry: &Entry, started_at: time::OffsetDateTime) -> Result<(), Error> {
        let database = &self.database;

        // Query the database to see if we have any song within this music folder that has the same
        // relative path. If yes, update its scanned at to the current time.
        let song_id = if let Some((song_id, updated_at)) = self.set_scanned_at(entry).await? {
            if entry.last_modified.is_some_and(|last_modified| last_modified < updated_at) {
                // If its filesystem's last modified is sooner than its database's updated at, it
                // means that we have the latest data, we can return the function.
                return Ok(());
            }
            Some(song_id)
        } else {
            None
        };

        let file = File::new(entry.format, self.filesystem.read(entry.path.to_path()).await?)?;
        let dir_picture_id = picture::Picture::scan(
            &self.database,
            &self.filesystem,
            &self.config.cover_art,
            entry
                .path
                .parent()
                .ok_or_else(|| error::Kind::MissingPathParent(entry.path.clone()))?,
        )
        .await?;

        let relative_path = self.relative_path(entry)?;
        let relative_path = relative_path.as_str();
        if let Some((database_song_id, database_relative_path)) =
            self.query_hash_size(&file.property).await?
        {
            if let Some(song_id) = song_id {
                if song_id == database_song_id && relative_path == database_relative_path {
                    // Everything is the same but the song's last modified for some reason, update
                    // its updated at and return the function.
                    diesel::update(songs::table)
                        .filter(songs::id.eq(song_id))
                        .set(songs::updated_at.eq(time::OffsetDateTime::now_utc()))
                        .execute(&mut database.get().await?)
                        .await?;
                    return Ok(());
                }
                // Since `song_id` is queried only by music folder and relative path and there is a
                // constraint `songs_album_id_file_hash_file_size_key`, other cases should be
                // unreachable.
                return error::Kind::DatabaseCorruptionDetected.into();
            }
            // We have one entry that is in the same music folder, same hash and size but
            // different relative path (since song_id is none). We only need to update the relative
            // path, set scanned at and return the function.
            let album_id: Uuid = diesel::update(songs::table)
                .filter(songs::id.eq(database_song_id))
                .set((
                    songs::relative_path.eq(relative_path),
                    songs::scanned_at.eq(time::OffsetDateTime::now_utc()),
                ))
                .returning(songs::album_id)
                .get_result(&mut database.get().await?)
                .await?;
            // We also need to set album cover_art_id since it might be added or removed after the
            // previous scan.
            diesel::update(albums::table)
                .filter(albums::id.eq(album_id))
                .set(albums::cover_art_id.eq(dir_picture_id))
                .execute(&mut database.get().await?)
                .await?;

            tracing::warn!(
                old = ?database_relative_path, new = ?relative_path, "renamed duplication"
            );
            return Ok(());
        }

        let audio = file.audio(self.config.lofty)?;
        let information = audio.extract(&self.config.parsing)?;

        let song_id = information
            .upsert(
                database,
                &self.config,
                albums::Foreign {
                    music_folder_id: self.music_folder.id,
                    cover_art_id: dir_picture_id,
                },
                relative_path,
                song_id,
            )
            .await?;
        audio::Information::cleanup_one(database, started_at, song_id).await?;

        Ok(())
    }

    #[instrument(skip_all, fields(started_at), err(Debug))]
    pub async fn run(&self) -> Result<(), Error> {
        let span = tracing::Span::current();
        let started_at = crate::time::now().await;
        span.record("started_at", tracing::field::display(&started_at));
        tracing::info!(music_folder = ?self.music_folder);

        let (scan_handle, permit, rx) = self.init();
        let mut join_set = tokio::task::JoinSet::new();

        while let Ok(entry) = rx.recv_async().await {
            let permit = permit.clone().acquire_owned().await?;
            let scan = self.clone().into_owned();
            join_set.spawn(
                async move {
                    let _guard = permit;
                    scan.one(&entry, started_at).await
                }
                .instrument(span.clone()),
            );
        }

        while let Some(result) = join_set.join_next().await {
            result??;
        }
        scan_handle.await??;

        audio::Information::cleanup(&self.database, started_at).await?;

        self.database.upsert_config(&self.config.index).await?;
        self.informant.search_and_upsert_artists(&self.database, &self.config.cover_art).await?;

        let latency: std::time::Duration =
            (time::OffsetDateTime::now_utc() - started_at).try_into()?;
        tracing::info!(took = ?latency);
        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::Fake;
    use rstest::rstest;

    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_simple_scan(#[future(awt)] mock: Mock, #[values(0, 10, 50)] n_song: usize) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().n_song(n_song).call().await;

        let database_audio = music_folder.query_filesystem().await;
        assert_eq!(database_audio, music_folder.filesystem);
    }

    // TODO: Make multiple scans work
    #[allow(dead_code)]
    async fn test_multiple_scan(mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().n_song(20).scan(false).call().await;

        let mut join_set = tokio::task::JoinSet::new();
        for _ in 0..5 {
            let scanner = music_folder.scan().into_owned();
            join_set.spawn(async move { scanner.run().await.unwrap() });
        }
        join_set.join_all().await;

        let database_audio = music_folder.query_filesystem().await;
        assert_eq!(database_audio, music_folder.filesystem);
    }

    mod filesystem {
        use super::*;

        #[rstest]
        #[tokio::test]
        async fn test_overwrite(#[future(awt)] mock: Mock) {
            let mut music_folder = mock.music_folder(0).await;

            music_folder.add_audio_filesystem().path("test").call().await;
            let database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio, music_folder.filesystem);

            music_folder.add_audio_filesystem().path("test").call().await;
            let database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio, music_folder.filesystem);
        }

        #[rstest]
        #[tokio::test]
        async fn test_remove(#[future(awt)] mock: Mock, #[values(true, false)] same_dir: bool) {
            let mut music_folder = mock.music_folder(0).await;
            music_folder
                .add_audio_filesystem::<&str>()
                .n_song(10)
                .depth(if same_dir { 0 } else { (1..3).fake() })
                .call()
                .await;
            music_folder.remove_audio_filesystem::<&str>().call().await;

            let database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio, music_folder.filesystem);
        }

        #[rstest]
        #[tokio::test]
        async fn test_duplicate(#[future(awt)] mock: Mock, #[values(true, false)] same_dir: bool) {
            let mut music_folder = mock.music_folder(0).await;
            music_folder.add_audio_filesystem::<&str>().depth(0).call().await;
            let audio = music_folder.filesystem[0].clone();

            music_folder
                .add_audio_filesystem::<&str>()
                .metadata(audio.information.metadata.clone())
                .format(audio.information.file.format)
                .depth(if same_dir { 0 } else { (1..3).fake() })
                .call()
                .await;

            let mut database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio.len(), 1);
            let (database_path, database_audio) = database_audio.shift_remove_index(0).unwrap();

            let (path, audio) = music_folder
                .filesystem
                .shift_remove_index(usize::from(
                    audio.relative_path != database_audio.relative_path,
                ))
                .unwrap();
            assert_eq!(database_path, path);

            let (database_audio, audio) = if same_dir {
                (database_audio, audio)
            } else {
                (database_audio.with_dir_picture(None), audio.with_dir_picture(None))
            };
            assert_eq!(database_audio, audio);
        }

        #[rstest]
        #[tokio::test]
        async fn test_move(#[future(awt)] mock: Mock) {
            let mut music_folder = mock.music_folder(0).await;
            music_folder.add_audio_filesystem::<&str>().call().await;
            let audio = music_folder.filesystem[0].clone();
            music_folder.remove_audio_filesystem::<&str>().index(0).call().await;

            music_folder
                .add_audio_filesystem::<&str>()
                .metadata(audio.information.metadata.clone())
                .format(audio.information.file.format)
                .call()
                .await;

            let database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio, music_folder.filesystem);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_scan_dir_picture(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder
            .add_audio_filesystem::<&str>()
            .n_song(10)
            .depth(0)
            .recompute_dir_picture(false)
            .call()
            .await;

        // All pictures are the same. However, the picture will only be the same from the first
        // file that has a picture so we have to filter out none before checking.
        let dir_pictures: Vec<_> = music_folder
            .filesystem
            .values()
            .filter_map(|information| information.dir_picture.clone())
            .collect();
        assert!(dir_pictures.windows(2).all(|window| window[0] == window[1]));

        // On the other hand, data queried from database should have all the same picture
        // regardless if the very first file have a picture or not. So we use `map` instead of
        // `filter_map` here.
        let database_dir_pictures: Vec<_> = music_folder
            .query_filesystem()
            .await
            .values()
            .map(|information| information.dir_picture.clone())
            .collect();
        assert!(database_dir_pictures.windows(2).all(|window| window[0] == window[1]));
    }
}
