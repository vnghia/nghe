use std::borrow::Cow;
use std::sync::Arc;

use diesel::{
    ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lofty::config::ParseOptions;
use loole::Receiver;
use nghe_api::scan;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tracing::{Instrument, instrument};
use typed_path::Utf8TypedPath;
use uuid::Uuid;

use crate::database::Database;
use crate::file::{self, File, audio, image, lyric};
use crate::filesystem::{self, Entry, Filesystem, Trait, entry};
use crate::integration::Informant;
use crate::orm::{albums, music_folders, songs};
use crate::{Error, config, error};

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
    pub full: scan::start::Full,
}

impl<'db, 'fs, 'mf> Scanner<'db, 'fs, 'mf> {
    #[coverage(off)]
    pub async fn new(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        config: Config,
        informant: Informant,
        request: scan::start::Request,
    ) -> Result<Self, Error> {
        Self::new_orm(
            database,
            filesystem,
            config,
            informant,
            music_folders::MusicFolder::query(database, request.music_folder_id).await?,
            request.full,
        )
    }

    pub fn new_orm(
        database: &'db Database,
        filesystem: &'fs Filesystem,
        config: Config,
        informant: Informant,
        music_folder: music_folders::MusicFolder<'mf>,
        full: scan::start::Full,
    ) -> Result<Self, Error> {
        let filesystem = filesystem.to_impl(music_folder.data.ty.into())?;
        Ok(Self {
            database: Cow::Borrowed(database),
            filesystem,
            config,
            informant,
            music_folder,
            full,
        })
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

    #[cfg_attr(not(coverage_nightly), instrument(skip_all, ret(level = "trace")))]
    async fn set_scanned_at(
        &self,
        entry: &Entry,
        started_at: time::OffsetDateTime,
    ) -> Result<Option<songs::IdTime>, Error> {
        let song_time = songs::table
            .inner_join(albums::table)
            .filter(albums::music_folder_id.eq(self.music_folder.id))
            .filter(
                songs::relative_path
                    .eq(entry.relative_path(&self.music_folder.data.path)?.as_str()),
            )
            .select(songs::IdTime::as_select())
            .get_result(&mut self.database.get().await?)
            .await
            .optional()?;

        // Only update `scanned_at` if it is sooner than `started_at`.
        // Else, it means that the current path is being scanned by another process or it is already
        // scanned.
        if let Some(song_time) = song_time
            && song_time.time.scanned_at < started_at
        {
            diesel::update(songs::table)
                .filter(songs::id.eq(song_time.id))
                .set(songs::scanned_at.eq(crate::time::now().await))
                .execute(&mut self.database.get().await?)
                .await?;
        }
        Ok(song_time)
    }

    #[cfg_attr(not(coverage_nightly), instrument(skip_all, ret(level = "trace")))]
    async fn query_hash_size(
        &self,
        property: &file::Property<audio::Format>,
    ) -> Result<Option<songs::IdPath>, Error> {
        songs::table
            .inner_join(albums::table)
            .filter(albums::music_folder_id.eq(self.music_folder.id))
            .filter(songs::file_hash.eq(property.hash.cast_signed()))
            .filter(songs::file_size.eq(property.size.get().cast_signed()))
            .select(songs::IdPath::as_select())
            .get_result(&mut self.database.get().await?)
            .await
            .optional()
            .map_err(Error::from)
    }

    async fn update_dir_picture(
        &self,
        song_id: Uuid,
        dir_picture_id: Option<Uuid>,
    ) -> Result<(), Error> {
        diesel::update(albums::table)
            .filter(albums::id.nullable().eq(
                songs::table.filter(songs::id.eq(song_id)).select(songs::album_id).single_value(),
            ))
            .set(albums::cover_art_id.eq(dir_picture_id))
            .execute(&mut self.database.get().await?)
            .await?;
        Ok(())
    }

    async fn update_external_lyric(
        &self,
        started_at: impl Into<Option<time::OffsetDateTime>>,
        song_id: Uuid,
        song_path: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        lyric::Lyric::scan(
            &self.database,
            &self.filesystem,
            self.full.external_lyric,
            song_id,
            song_path,
        )
        .await?;
        if let Some(started_at) = started_at.into() {
            lyric::Lyric::cleanup_one_external(&self.database, started_at, song_id).await?;
        }
        Ok(())
    }

    async fn update_external(
        &self,
        started_at: time::OffsetDateTime,
        song_id: Uuid,
        song_path: Utf8TypedPath<'_>,
        dir_picture_id: Option<Uuid>,
    ) -> Result<(), Error> {
        // We also need to set album cover_art_id and external lyrics since it might be
        // added or removed after the previous scan.
        self.update_dir_picture(song_id, dir_picture_id).await?;
        self.update_external_lyric(started_at, song_id, song_path).await?;
        Ok(())
    }

    #[cfg_attr(
        not(coverage_nightly),
        instrument(
            skip_all,
            fields(path = %entry.path, last_modified = ?entry.last_modified),
            ret(level = "debug"),
            err(Debug)
        )
    )]
    async fn one(&self, entry: &Entry, started_at: time::OffsetDateTime) -> Result<Uuid, Error> {
        let database = &self.database;

        // Query the database to see if we have any song within this music folder that has the same
        // relative path. If yes, update its scanned at to the current time.
        //
        // Doing this helps us avoiding working on the same file at the same time (which is mostly
        // the case for multiple scans).
        let song_id = if let Some(song_time) = self.set_scanned_at(entry, started_at).await? {
            if started_at < song_time.time.scanned_at
                || (!self.full.file
                    && entry
                        .last_modified
                        .is_some_and(|last_modified| last_modified < song_time.time.updated_at))
            {
                // If `started_at` is sooner than its database's `scanned_at` or its filesystem's
                // last modified is sooner than its database's `updated_at`, it means that we have
                // the latest data or this file is being scanned by another process, we can return
                // the function.
                //
                // Since the old `scanned_at` is returned, there is a case when the file is scanned
                // in the previous scan but not in the current scan, thus `scanned_at` is sooner
                // than `started_at`. We want to skip this file as well (unless in full mode) hence
                // we have to check for its `last_modified` along with `scanned_at`.
                return Ok(song_time.id);
            }
            Some(song_time.id)
        } else {
            None
        };

        let absolute_path = entry.path.to_path();
        let file = File::new(entry.format, self.filesystem.read(absolute_path).await?)?;
        let dir_picture_id = image::Image::scan(
            &self.database,
            &self.filesystem,
            &self.config.cover_art,
            self.full.dir_picture,
            entry
                .path
                .parent()
                .ok_or_else(|| error::Kind::MissingPathParent(entry.path.clone()))?,
        )
        .await?;
        tracing::trace!(?dir_picture_id);

        let relative_path = self.relative_path(entry)?;
        let relative_path = relative_path.as_str();
        let song_id = if let Some(song_path) = self.query_hash_size(&file.property).await? {
            if started_at < song_path.time.updated_at {
                // We will check if `song_path.updated_at` is later than `started_at`, since this
                // file has the same hash and size with that entry in the database, we can terminate
                // this function regardless of full mode as another file with the same data is
                // processed in the current scan.
                //
                // `song_id` can be None if there are more than two duplicated files in the same
                // music folder.

                self.update_external(started_at, song_path.id, absolute_path, dir_picture_id)
                    .await?;
                tracing::debug!("already scanned");
                return Ok(song_path.id);
            } else if let Some(song_id) = song_id {
                // `DatabaseCorruption` can happen if all the below conditions hold:
                //  - There is a file on the filesystem that has the same hash and size as those of
                //    one entry in the database (`hash_size` constraint) but not the same relative
                //    (P_fs and P_db) path. Could be the result of a duplication or renaming
                //    operation.
                //  - The file with P_fs is scanned first and update the relative path in the
                //    database to P_fs (thread 1).
                //  - The file with P_db is scanned before the relative path is updated to P_fs
                //    therefore it still returns an entry (thread 2).
                //  - However, `query_hash_size` operation of thread 2 takes place after the update
                //    of relative path by thread 1, thus causing the `DatabaseCorruption` error as
                //    `relative_path != song_path.relative_path`.
                //
                // We prevent this error by checking the `song_path.updated_at` as above so we can
                // skip checking `song_path.relative_path` after being updated.
                if song_id == song_path.id && relative_path == song_path.relative_path {
                    if self.full.file {
                        // If file full scan is enabled, we return the song id so it can be
                        // re-scanned later.
                        Some(song_path.id)
                    } else {
                        // Everything is the same but the song's last modified for some reason,
                        // update its updated at and return the function.
                        diesel::update(songs::table)
                            .filter(songs::id.eq(song_id))
                            .set(songs::updated_at.eq(crate::time::now().await))
                            .execute(&mut database.get().await?)
                            .await?;

                        self.update_external(
                            started_at,
                            song_path.id,
                            absolute_path,
                            dir_picture_id,
                        )
                        .await?;
                        tracing::debug!("stale last_modified");
                        return Ok(song_path.id);
                    }
                } else {
                    // Since `song_id` is queried only by music folder and relative path and there
                    // is a constraint `songs_album_id_file_hash_file_size_key`,
                    // other cases should be unreachable.
                    return error::Kind::DatabaseCorruptionDetected.into();
                }
            } else if self.full.file {
                // If file full scan is enabled, we return the song id so it can be
                // re-scanned later. Here `song_id` is None.
                Some(song_path.id)
            } else {
                // We have one entry that is in the same music folder, same hash and size but
                // different relative path (since song_id is None). We only need to update the
                // relative path, set scanned at and return the function.
                diesel::update(songs::table)
                    .filter(songs::id.eq(song_path.id))
                    .set((
                        songs::relative_path.eq(relative_path),
                        songs::scanned_at.eq(crate::time::now().await),
                    ))
                    .execute(&mut database.get().await?)
                    .await?;

                self.update_external(started_at, song_path.id, absolute_path, dir_picture_id)
                    .await?;
                tracing::warn!(
                    old = %song_path.relative_path, new = %relative_path, "renamed duplication"
                );
                return Ok(song_path.id);
            }
        } else {
            song_id
        };

        let audio = file.audio(self.config.lofty)?;
        let information = audio.extract(&self.config.parsing)?;
        tracing::trace!(?information);

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
        self.update_external_lyric(None, song_id, absolute_path).await?;
        audio::Information::cleanup_one(database, started_at, song_id).await?;

        Ok(song_id)
    }

    #[cfg_attr(not(coverage_nightly), instrument(skip_all, fields(started_at), err(Debug)))]
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
        self.informant
            .search_and_upsert_artists(
                &self.database,
                &self.config.cover_art,
                self.full.information,
            )
            .await?;

        let latency: std::time::Duration =
            (time::OffsetDateTime::now_utc() - started_at).try_into()?;
        tracing::info!(took = ?latency);
        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use nghe_api::scan;
    use rstest::rstest;

    use crate::file::audio;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_simple_scan(#[future(awt)] mock: Mock, #[values(0, 10, 50)] n_song: usize) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().n_song(n_song).call().await;

        let database_audio = music_folder.query_filesystem().await;
        assert_eq!(database_audio, music_folder.filesystem);
    }

    #[rstest]
    #[tokio::test]
    async fn test_full_scan(#[future(awt)] mock: Mock, #[values(true, false)] full: bool) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().call().await;

        let song_id = music_folder.song_id_filesystem(0).await;
        let filesystem_audio = music_folder.filesystem[0].clone();
        // Don't modify lyric because we won't rescan it even in full mode (it will be rescanned
        // only in full lyric mode).
        music_folder
            .add_audio()
            .album(filesystem_audio.information.metadata.album)
            .file_property(filesystem_audio.information.file)
            .external_lyric(None)
            .relative_path(filesystem_audio.relative_path)
            .song_id(song_id)
            .call()
            .await;
        music_folder
            .scan(scan::start::Full { file: full, ..Default::default() })
            .run()
            .await
            .unwrap();

        let database_audio = music_folder.query_filesystem().await;
        if full {
            assert_eq!(database_audio, music_folder.filesystem);
        } else {
            // Could not compare information that uses more than one table.
            assert_eq!(
                database_audio[0].information.metadata.song,
                music_folder.database[0].information.metadata.song
            );
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_multiple_scan(#[future(awt)] mock: Mock) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().n_song(20).scan(false).call().await;

        let mut join_set = tokio::task::JoinSet::new();
        for _ in 0..5 {
            let scanner = music_folder.scan(scan::start::Full::default()).into_owned();
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
        async fn test_overwrite(
            #[future(awt)] mock: Mock,
            #[values(true, false)] same_album: bool,
            #[values(true, false)] same_external_lyric: bool,
        ) {
            // Test a constraint with `album_id` and `relative_path`.
            let mut music_folder = mock.music_folder(0).await;
            let album: audio::Album = Faker.fake();

            music_folder.add_audio_filesystem().album(album.clone()).path("test").call().await;
            let database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio, music_folder.filesystem);

            music_folder
                .add_audio_filesystem()
                .maybe_album(if same_album { Some(album) } else { None })
                .maybe_external_lyric(if same_external_lyric {
                    Some(database_audio[0].external_lyric.clone())
                } else {
                    None
                })
                .path("test")
                .format(database_audio[0].information.file.format)
                .call()
                .await;

            let mut database_audio = music_folder.query_filesystem().await;
            assert_eq!(database_audio.len(), 1);
            assert_eq!(music_folder.filesystem.len(), 1);

            let database_audio = database_audio.shift_remove_index(0).unwrap().1;
            let filesystem_audio = music_folder.filesystem.shift_remove_index(0).unwrap().1;

            let (database_audio, filesystem_audio) = if same_external_lyric {
                (database_audio, filesystem_audio)
            } else {
                (
                    database_audio.with_external_lyric(None),
                    filesystem_audio.with_external_lyric(None),
                )
            };

            let (database_audio, filesystem_audio) = if same_album {
                (database_audio.with_dir_picture(None), filesystem_audio.with_dir_picture(None))
            } else {
                (database_audio, filesystem_audio)
            };

            assert_eq!(database_audio, filesystem_audio);
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
        async fn test_duplicate(
            #[future(awt)] mock: Mock,
            #[values(true, false)] same_dir: bool,
            #[values(true, false)] same_external_lyric: bool,
            #[values(true, false)] full: bool,
        ) {
            let mut music_folder = mock.music_folder(0).await;
            music_folder.add_audio_filesystem::<&str>().depth(0).call().await;
            let audio = music_folder.filesystem[0].clone();

            music_folder
                .add_audio_filesystem::<&str>()
                .metadata(audio.information.metadata.clone())
                .maybe_external_lyric(if same_external_lyric {
                    Some(audio.external_lyric.clone())
                } else {
                    None
                })
                .format(audio.information.file.format)
                .depth(if same_dir { 0 } else { (1..3).fake() })
                .full(scan::start::Full { file: full, ..Default::default() })
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

            let (database_audio, audio) = if same_external_lyric {
                (database_audio, audio)
            } else {
                (database_audio.with_external_lyric(None), audio.with_external_lyric(None))
            };

            let (database_audio, audio) = if same_dir {
                (database_audio, audio)
            } else {
                (database_audio.with_dir_picture(None), audio.with_dir_picture(None))
            };
            assert_eq!(database_audio, audio);
        }

        #[rstest]
        #[tokio::test]
        async fn test_move(#[future(awt)] mock: Mock, #[values(true, false)] full: bool) {
            let mut music_folder = mock.music_folder(0).await;
            music_folder.add_audio_filesystem::<&str>().call().await;
            let audio = music_folder.filesystem[0].clone();
            music_folder.remove_audio_filesystem::<&str>().index(0).call().await;

            music_folder
                .add_audio_filesystem::<&str>()
                .metadata(audio.information.metadata.clone())
                .format(audio.information.file.format)
                .full(scan::start::Full { file: full, ..Default::default() })
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
