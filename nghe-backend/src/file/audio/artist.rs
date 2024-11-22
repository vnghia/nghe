use std::borrow::{Borrow, Cow};

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use futures_lite::{stream, StreamExt};
use indexmap::IndexSet;
use o2o::o2o;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::upsert::Insert as _;
use crate::orm::{artists, songs_album_artists, songs_artists};
use crate::Error;

#[derive(Debug, PartialEq, Eq, Hash, o2o)]
#[from_owned(artists::Data<'a>)]
#[ref_into(artists::Data<'a>)]
#[cfg_attr(test, derive(Dummy, Clone, PartialOrd, Ord))]
pub struct Artist<'a> {
    #[ref_into(Cow::Borrowed(~.as_ref()))]
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Dummy, Eq, Clone))]
pub struct Artists<'a> {
    #[cfg_attr(test, dummy(expr = "fake::vec![Artist; 1..5].into_iter().collect()"))]
    pub song: IndexSet<Artist<'a>>,
    #[cfg_attr(test, dummy(expr = "fake::vec![Artist; 0..3].into_iter().collect()"))]
    pub album: IndexSet<Artist<'a>>,
    pub compilation: bool,
}

impl<'a> Artist<'a> {
    pub fn index(&self, prefixes: &[impl AsRef<str>]) -> Result<char, Error> {
        let mut iter = prefixes.iter();
        let name = loop {
            match iter.next() {
                Some(prefix) => {
                    if let Some(name) = self.name.strip_prefix(prefix.as_ref()) {
                        break name;
                    }
                }
                None => break self.name.as_ref(),
            }
        };
        name.nfkd().next().ok_or_else(|| Error::MediaArtistNameEmpty).map(|c| {
            if c.is_ascii_alphabetic() {
                c.to_ascii_uppercase()
            } else if c.is_numeric() {
                '#'
            } else if !c.is_alphabetic() {
                '*'
            } else {
                c
            }
        })
    }

    pub async fn upsert(
        &self,
        database: &Database,
        prefixes: &[impl AsRef<str>],
    ) -> Result<Uuid, Error> {
        artists::Upsert { index: self.index(prefixes)?.to_string().into(), data: self.into() }
            .insert(database)
            .await
    }

    async fn upserts<S: Borrow<Self> + 'a>(
        database: &Database,
        artists: impl IntoIterator<Item = S>,
        prefixes: &[impl AsRef<str>],
    ) -> Result<Vec<Uuid>, Error> {
        stream::iter(artists)
            .then(async |artist| artist.borrow().upsert(database, prefixes).await)
            .try_collect()
            .await
    }
}

impl<'a> Artists<'a> {
    pub fn new(
        song: impl IntoIterator<Item = Artist<'a>>,
        album: impl IntoIterator<Item = Artist<'a>>,
        compilation: bool,
    ) -> Result<Self, Error> {
        let song: IndexSet<_> = song.into_iter().collect();
        let album = album.into_iter().collect();
        if song.is_empty() {
            Err(Error::MediaSongArtistEmpty)
        } else {
            Ok(Self { song, album, compilation })
        }
    }

    pub fn song(&self) -> &IndexSet<Artist<'a>> {
        &self.song
    }

    pub fn album(&self) -> &IndexSet<Artist<'a>> {
        if self.album.is_empty() { &self.song } else { &self.album }
    }

    pub fn compilation(&self) -> bool {
        // If the song has compilation, all artists in the song artists will be added to song
        // album artists with compilation set to true. If it also contains any album
        // artists, the compilation field will be overwritten to false later. If the
        // album artists field is empty, the album artists will be the same with
        // song artists which then set any compilation field to false, so no need to add them in
        // the first place.
        if self.album.is_empty() || self.song.difference(&self.album).peekable().peek().is_none() {
            false
        } else {
            self.compilation
        }
    }

    pub async fn upsert_song_artist(
        database: &Database,
        song_id: Uuid,
        artist_ids: &[Uuid],
    ) -> Result<(), Error> {
        diesel::insert_into(songs_artists::table)
            .values::<Vec<_>>(
                artist_ids
                    .iter()
                    .copied()
                    .map(|artist_id| songs_artists::Data { song_id, artist_id })
                    .collect(),
            )
            .on_conflict((songs_artists::song_id, songs_artists::artist_id))
            .do_update()
            .set(songs_artists::upserted_at.eq(time::OffsetDateTime::now_utc()))
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }

    pub async fn upsert_song_album_artist(
        database: &Database,
        song_id: Uuid,
        album_artist_ids: &[Uuid],
        compilation: bool,
    ) -> Result<(), Error> {
        diesel::insert_into(songs_album_artists::table)
            .values::<Vec<_>>(
                album_artist_ids
                    .iter()
                    .copied()
                    .map(|album_artist_id| songs_album_artists::Data {
                        song_id,
                        album_artist_id,
                        compilation,
                    })
                    .collect(),
            )
            .on_conflict((songs_album_artists::song_id, songs_album_artists::album_artist_id))
            .do_update()
            .set((
                songs_album_artists::compilation.eq(compilation),
                songs_album_artists::upserted_at.eq(time::OffsetDateTime::now_utc()),
            ))
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }

    pub async fn upsert(
        &self,
        database: &Database,
        prefixes: &[impl AsRef<str>],
        song_id: Uuid,
    ) -> Result<(), Error> {
        let song_artist_ids = Artist::upserts(database, &self.song, prefixes).await?;
        Self::upsert_song_artist(database, song_id, &song_artist_ids).await?;
        if self.compilation() {
            // If the song has compilation, all artists in the song artists will be added to song
            // album artists with compilation set to true.
            Self::upsert_song_album_artist(database, song_id, &song_artist_ids, true).await?;
        }

        // If there isn't any album artist,
        // we assume that they are the same as artists.
        let album_artist_ids = if self.album.is_empty() {
            song_artist_ids
        } else {
            Artist::upserts(database, &self.album, prefixes).await?
        };
        Self::upsert_song_album_artist(database, song_id, &album_artist_ids, false).await?;

        Ok(())
    }

    pub async fn cleanup_one(
        database: &Database,
        started_at: time::OffsetDateTime,
        song_id: Uuid,
    ) -> Result<(), Error> {
        // Delete all artists of a song which haven't been refreshed since timestamp.
        diesel::delete(songs_artists::table)
            .filter(songs_artists::song_id.eq(song_id))
            .filter(songs_artists::upserted_at.lt(started_at))
            .execute(&mut database.get().await?)
            .await?;

        // Delete all album artists of a song which haven't been refreshed since timestamp.
        diesel::delete(songs_album_artists::table)
            .filter(songs_album_artists::song_id.eq(song_id))
            .filter(songs_album_artists::upserted_at.lt(started_at))
            .execute(&mut database.get().await?)
            .await?;

        Ok(())
    }

    pub async fn cleanup(database: &Database) -> Result<(), Error> {
        // Delete all artists which does not have any relation with an album
        // (via songs_album_artists) or a song (via songs_artists).
        let alias_artists = diesel::alias!(artists as alias_artists);
        diesel::delete(artists::table)
            .filter(
                artists::id.eq_any(
                    alias_artists
                        .filter(not(exists(
                            songs_album_artists::table.filter(
                                songs_album_artists::album_artist_id
                                    .eq(alias_artists.field(artists::id)),
                            ),
                        )))
                        .filter(not(exists(songs_artists::table.filter(
                            songs_artists::artist_id.eq(alias_artists.field(artists::id)),
                        ))))
                        .select(alias_artists.field(artists::id)),
                ),
            )
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use diesel::{QueryDsl, SelectableHelper};
    use itertools::Itertools;

    use super::*;
    use crate::orm::songs;
    use crate::test::Mock;

    impl<S: AsRef<str> + Sized> PartialEq<S> for Artist<'_> {
        fn eq(&self, other: &S) -> bool {
            self.name == other.as_ref() && self.mbz_id.is_none()
        }
    }

    impl PartialEq for Artists<'_> {
        fn eq(&self, other: &Self) -> bool {
            (self.song() == other.song())
                && (self.album() == other.album())
                && (self.compilation() == other.compilation())
        }
    }

    impl<'a> From<&'a str> for Artist<'a> {
        fn from(value: &'a str) -> Self {
            Self { name: value.into(), mbz_id: None }
        }
    }

    impl From<String> for Artist<'static> {
        fn from(value: String) -> Self {
            Self { name: value.into(), mbz_id: None }
        }
    }

    impl<'a> From<(&'a str, Uuid)> for Artist<'a> {
        fn from(value: (&'a str, Uuid)) -> Self {
            Self { name: value.0.into(), mbz_id: Some(value.1) }
        }
    }

    impl Artist<'_> {
        pub async fn upsert_mock(&self, mock: &Mock) -> Uuid {
            self.upsert(mock.database(), &mock.config.index.ignore_prefixes).await.unwrap()
        }
    }

    impl Artist<'static> {
        pub async fn query(mock: &Mock, id: Uuid) -> Self {
            artists::table
                .filter(artists::id.eq(id))
                .select(artists::Data::as_select())
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
                .into()
        }

        async fn query_ids(mock: &Mock, ids: &[Uuid], sorted: bool) -> Vec<Self> {
            let artists: Vec<_> = stream::iter(ids)
                .copied()
                .then(async |id| Self::query(mock, id).await)
                .collect()
                .await;
            if sorted { artists.into_iter().sorted().collect() } else { artists }
        }

        async fn query_song_artists(mock: &Mock, song_id: Uuid) -> (Vec<Uuid>, Vec<Self>) {
            let ids: Vec<Uuid> = songs_artists::table
                .filter(songs_artists::song_id.eq(song_id))
                .select(songs_artists::artist_id)
                .order_by(songs_artists::upserted_at)
                .get_results(&mut mock.get().await)
                .await
                .unwrap();
            let artists = Self::query_ids(mock, &ids, false).await;
            (ids, artists)
        }

        async fn query_song_album_artists(
            mock: &Mock,
            song_id: Uuid,
            artist_ids: &[Uuid],
        ) -> (Vec<Self>, bool) {
            let ids_compilations = songs_album_artists::table
                .filter(songs_album_artists::song_id.eq(song_id))
                .select((songs_album_artists::album_artist_id, songs_album_artists::compilation))
                .order_by(songs_album_artists::upserted_at)
                .get_results::<(Uuid, bool)>(&mut mock.get().await)
                .await
                .unwrap();
            let artists: Vec<_> = stream::iter(&ids_compilations)
                .copied()
                .filter_map(|(id, compilation)| {
                    if compilation {
                        assert!(
                            artist_ids.contains(&id),
                            "Stale compilation album artist has not been removed yet"
                        );
                        None
                    } else {
                        Some(id)
                    }
                })
                .then(async |id| Self::query(mock, id).await)
                .collect()
                .await;
            // If there is any compliation, it will be filtered out and make the size of two vectors
            // not equal. On the other hand, two same size vectors can mean either there
            // isn't any compilation or the song artists are the same as the album
            // artists or there isn't any album artist (which then be filled with song
            // artists).
            let compilation = ids_compilations.len() != artists.len();
            (artists, compilation)
        }

        pub async fn query_album(mock: &Mock, album_id: Uuid) -> Vec<Self> {
            let ids = songs_album_artists::table
                .inner_join(songs::table)
                .select(songs_album_artists::album_artist_id)
                .filter(songs::album_id.eq(album_id))
                .get_results(&mut mock.get().await)
                .await
                .unwrap();
            Self::query_ids(mock, &ids, true).await
        }

        pub async fn queries(mock: &Mock) -> Vec<Self> {
            let ids = artists::table
                .select(artists::id)
                .get_results(&mut mock.get().await)
                .await
                .unwrap();
            Self::query_ids(mock, &ids, true).await
        }
    }

    impl Artists<'static> {
        pub async fn query(mock: &Mock, song_id: Uuid) -> Self {
            let (artist_ids, song) = Artist::query_song_artists(mock, song_id).await;
            let (album, compilation) =
                Artist::query_song_album_artists(mock, song_id, &artist_ids).await;
            Self::new(song, album, compilation).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[case("The One", &["The ", "A "], 'O')]
    #[case("The 1", &["The ", "A "], '#')]
    #[case("The one", &["The ", "A "], 'O')]
    #[case("狼", &["The ", "A "], '狼')]
    #[case("é", &["The ", "A "], 'E')]
    #[case("ド", &["The ", "A "], 'ト')]
    #[case("ａ", &["The ", "A "], 'A')]
    #[case("%", &["The ", "A "], '*')]
    fn test_index(#[case] name: &str, #[case] prefixes: &[&str], #[case] index: char) {
        assert_eq!(Artist::from(name).index(prefixes).unwrap(), index);
    }

    #[rstest]
    #[case(&["Song"], &["Album"], true, true)]
    #[case(&["Song"], &["Album"], false, false)]
    #[case(&["Song"], &[], true, false)]
    #[case(&["Song"], &[], false, false)]
    #[case(&["Song"], &["Song"], true, false)]
    #[case(&["Song"], &["Song"], false, false)]
    #[case(&["Song"], &["Song", "Album"], true, false)]
    #[case(&["Song"], &["Song", "Album"], false, false)]
    #[case(&["Song1", "Song2"], &["Song1", "Song2", "Album"], true, false)]
    #[case(&["Song1", "Song2"], &["Song1", "Song2", "Album"], false, false)]
    #[case(&["Song1", "Song2"], &["Song2", "Album", "Song1"], true, false)]
    #[case(&["Song1", "Song2"], &["Song2", "Album", "Song1"], false, false)]
    fn test_compilation(
        #[case] song: &[&str],
        #[case] album: &[&str],
        #[case] compilation: bool,
        #[case] result: bool,
    ) {
        assert_eq!(
            Artists::new(
                song.iter().copied().map(Artist::from),
                album.iter().copied().map(Artist::from),
                compilation
            )
            .unwrap()
            .compilation(),
            result
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_artist_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] mbz_id: bool,
        #[values(true, false)] update_artist: bool,
    ) {
        let mbz_id = if mbz_id { Some(Faker.fake()) } else { None };
        let artist = Artist { mbz_id, ..Faker.fake() };
        let id = artist.upsert_mock(&mock).await;
        let database_artist = Artist::query(&mock, id).await;
        assert_eq!(database_artist, artist);

        if update_artist {
            let update_artist = Artist { mbz_id, ..Faker.fake() };
            let update_id = update_artist.upsert_mock(&mock).await;
            let database_update_artist = Artist::query(&mock, id).await;
            if mbz_id.is_some() {
                assert_eq!(id, update_id);
                assert_eq!(database_update_artist, update_artist);
            } else {
                // This will always insert a new row to the database
                // since there is nothing to identify an old artist.
                assert_ne!(id, update_id);
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_artist_upsert_no_mbz_id(#[future(awt)] mock: Mock) {
        // We want to make sure that insert the same artist with no mbz_id
        // twice does not result in any error.
        let artist = Artist { mbz_id: None, ..Faker.fake() };
        let id = artist.upsert_mock(&mock).await;
        let update_id = artist.upsert_mock(&mock).await;
        assert_eq!(update_id, id);
    }

    #[rstest]
    #[tokio::test]
    async fn test_artists_upsert(
        #[future(awt)] mock: Mock,
        #[values(true, false)] compilation: bool,
        #[values(true, false)] update_artists: bool,
    ) {
        let database = mock.database();
        let prefixes = &mock.config.index.ignore_prefixes;

        let information: audio::Information = Faker.fake();
        let album_id = information.metadata.album.upsert_mock(&mock, 0).await;
        let song_id = information
            .upsert_song(database, album_id.into(), Faker.fake::<String>(), None)
            .await
            .unwrap();

        let artists = Artists { compilation, ..Faker.fake() };
        artists.upsert(database, prefixes, song_id).await.unwrap();
        let database_artists = Artists::query(&mock, song_id).await;
        assert_eq!(database_artists, artists);

        if update_artists {
            let timestamp = crate::time::now().await;

            let update_artists = Artists { compilation, ..Faker.fake() };
            update_artists.upsert(database, prefixes, song_id).await.unwrap();
            Artists::cleanup_one(database, timestamp, song_id).await.unwrap();
            let database_update_artists = Artists::query(&mock, song_id).await;
            assert_eq!(database_update_artists, update_artists);
        }
    }

    mod cleanup {
        use super::*;

        #[rstest]
        #[tokio::test]
        async fn test_artist_all(#[future(awt)] mock: Mock) {
            let mut music_folder = mock.music_folder(0).await;
            music_folder.add_audio().n_song(5).call().await;
            assert!(!Artist::queries(&mock).await.is_empty());

            diesel::delete(songs_artists::table).execute(&mut mock.get().await).await.unwrap();
            diesel::delete(songs_album_artists::table)
                .execute(&mut mock.get().await)
                .await
                .unwrap();

            Artists::cleanup(mock.database()).await.unwrap();
            assert!(Artist::queries(&mock).await.is_empty());
        }

        #[rstest]
        #[case(1, 0)]
        #[case(1, 1)]
        #[case(5, 3)]
        #[case(5, 5)]
        #[tokio::test]
        async fn test_artist_song(
            #[future(awt)] mock: Mock,
            #[case] n_song: usize,
            #[case] n_subset: usize,
            #[values(true, false)] compilation: bool,
        ) {
            let mut music_folder = mock.music_folder(0).await;
            let artist: Artist = Faker.fake();
            music_folder
                .add_audio_artist(
                    [artist.clone(), Faker.fake()],
                    [Faker.fake()],
                    compilation,
                    n_song,
                )
                .await;
            let song_ids: Vec<_> = music_folder.database.keys().collect();
            assert!(Artist::queries(&mock).await.contains(&artist));

            diesel::delete(songs_artists::table)
                .filter(songs_artists::song_id.eq_any(&song_ids[0..n_subset]))
                .execute(&mut mock.get().await)
                .await
                .unwrap();
            if compilation {
                diesel::delete(songs_album_artists::table)
                    .filter(songs_album_artists::song_id.eq_any(&song_ids[0..n_subset]))
                    .execute(&mut mock.get().await)
                    .await
                    .unwrap();
            }
            Artists::cleanup(mock.database()).await.unwrap();
            assert_eq!(Artist::queries(&mock).await.contains(&artist), n_subset < n_song);
        }

        #[rstest]
        #[case(1, 0)]
        #[case(1, 1)]
        #[case(5, 3)]
        #[case(5, 5)]
        #[tokio::test]
        async fn test_artist_album(
            #[future(awt)] mock: Mock,
            #[case] n_album: usize,
            #[case] n_subset: usize,
        ) {
            let artist: Artist = Faker.fake();
            let album_song_ids: Vec<(Uuid, Vec<_>)> = stream::iter(0..n_album)
                .then(async |_| {
                    let mut music_folder = mock.music_folder(0).await;
                    let album: audio::Album = Faker.fake();
                    let album_id = album.upsert_mock(&mock, 0).await;
                    music_folder
                        .add_audio_artist(
                            [Faker.fake()],
                            [artist.clone(), Faker.fake()],
                            false,
                            (1..3).fake(),
                        )
                        .await;
                    let song_ids = music_folder.database.keys().copied().collect();
                    (album_id, song_ids)
                })
                .collect()
                .await;
            assert!(Artist::queries(&mock).await.contains(&artist));

            diesel::delete(songs_album_artists::table)
                .filter(
                    songs_album_artists::song_id.eq_any(
                        album_song_ids[0..n_subset]
                            .iter()
                            .flat_map(|(_, song_ids)| song_ids.clone())
                            .collect::<Vec<_>>(),
                    ),
                )
                .execute(&mut mock.get().await)
                .await
                .unwrap();
            Artists::cleanup(mock.database()).await.unwrap();
            assert_eq!(Artist::queries(&mock).await.contains(&artist), n_subset < n_album);
        }
    }
}
