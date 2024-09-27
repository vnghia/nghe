use std::borrow::Cow;

use diesel::ExpressionMethods;
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
#[cfg_attr(test, derive(Dummy, Clone))]
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

    async fn upserts(
        database: &Database,
        artists: impl IntoIterator<Item = &'a Self>,
        prefixes: &[impl AsRef<str>],
    ) -> Result<Vec<Uuid>, Error> {
        stream::iter(artists)
            .then(async |artist| artist.upsert(database, prefixes).await)
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
}

#[cfg(test)]
mod test {
    use super::*;

    impl<'a> PartialEq for Artists<'a> {
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

    impl<'a> From<(&'a str, Uuid)> for Artist<'a> {
        fn from(value: (&'a str, Uuid)) -> Self {
            Self { name: value.0.into(), mbz_id: Some(value.1) }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

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
}
