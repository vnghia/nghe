use std::borrow::Cow;
use std::str::FromStr;

#[cfg(test)]
use fake::{Dummy, Fake};
use isolang::Language;
#[cfg(test)]
use itertools::Itertools;
use o2o::o2o;

use super::{artist, name_date_mbz, position};
use crate::orm::songs;
use crate::Error;

#[derive(Debug, o2o)]
#[try_map_owned(songs::Song<'a>, Error)]
#[ref_try_into(songs::Song<'a>, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Song<'a> {
    #[map_owned(~.try_into()?)]
    #[ref_into((&~).try_into()?)]
    pub main: name_date_mbz::NameDateMbz<'a>,
    #[map(~.try_into()?)]
    pub track_disc: position::TrackDisc,
    #[from(~.into_iter().map(
        |language|Language::from_str(language.ok_or_else(
            || Error::LanguageFromDatabaseIsNull)?.as_ref()
        ).map_err(Error::from)
    ).try_collect()?)]
    #[into(~.iter().map(|language| Some(language.to_639_3().into())).collect())]
    #[cfg_attr(
        test,
        dummy(expr = "((0..=7915), \
                      0..=2).fake::<Vec<usize>>().into_iter().unique().\
                      map(Language::from_usize).collect::<Option<_>>().unwrap()")
    )]
    pub languages: Vec<Language>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Metadata<'a> {
    pub song: Song<'a>,
    pub album: name_date_mbz::NameDateMbz<'a>,
    pub artists: artist::Artists<'a>,
    #[cfg_attr(
        test,
        dummy(expr = "fake::vec![String; 0..=2].into_iter().map(String::into).collect()")
    )]
    pub genres: Vec<Cow<'a, str>>,
}

#[cfg(test)]
mod test {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use fake::Faker;
    use rstest::rstest;
    use uuid::Uuid;

    use super::*;
    use crate::file;
    use crate::file::audio::{self, Audio};
    use crate::orm::albums;
    use crate::test::{mock, Mock};

    async fn upsert_album(
        mock: &Mock,
        id: impl Into<Option<Uuid>>,
    ) -> (Uuid, name_date_mbz::NameDateMbz<'static>) {
        let id = id.into();
        let album: name_date_mbz::NameDateMbz = Faker.fake();
        let upsert = albums::Upsert {
            music_folder_id: mock.music_folder(0).await.music_folder.id,
            data: (&album).try_into().unwrap(),
        };
        let id = if let Some(id) = id {
            diesel::update(albums::table)
                .filter(albums::id.eq(id))
                .set(upsert)
                .execute(&mut mock.get().await)
                .await
                .unwrap();
            id
        } else {
            diesel::insert_into(albums::table)
                .values(upsert)
                .returning(albums::id)
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
        };
        (id, album)
    }

    async fn upsert_song(mock: &Mock, id: impl Into<Option<Uuid>>) -> (Uuid, Audio) {
        let id = id.into();
        let audio: Audio = Faker.fake();
        let (album_id, _) = upsert_album(mock, None).await;
        let upsert = songs::Upsert {
            album_id,
            relative_path: Faker.fake::<String>().into(),
            data: (&audio).try_into().unwrap(),
        };
        let id = if let Some(id) = id {
            diesel::update(songs::table)
                .filter(songs::id.eq(id))
                .set(upsert)
                .execute(&mut mock.get().await)
                .await
                .unwrap();
            id
        } else {
            diesel::insert_into(songs::table)
                .values(upsert)
                .returning(songs::id)
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
        };
        (id, audio)
    }

    async fn select_album(mock: &Mock, id: Uuid) -> name_date_mbz::NameDateMbz {
        albums::table
            .filter(albums::id.eq(id))
            .select(albums::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap()
            .try_into()
            .unwrap()
    }

    async fn select_song(mock: &Mock, id: Uuid) -> songs::Data {
        songs::table
            .filter(songs::id.eq(id))
            .select(songs::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_album_roundtrip(#[future(awt)] mock: Mock) {
        let (id, album) = upsert_album(&mock, None).await;
        let database_album = select_album(&mock, id).await;
        assert_eq!(database_album, album);
    }

    #[rstest]
    #[tokio::test]
    async fn test_album_update_roundtrip(#[future(awt)] mock: Mock) {
        let (id, _) = upsert_album(&mock, None).await;
        let (id, album) = upsert_album(&mock, id).await;
        let database_album = select_album(&mock, id).await;
        assert_eq!(database_album, album);
    }

    #[rstest]
    #[tokio::test]
    async fn test_song_roundtrip(#[future(awt)] mock: Mock) {
        let (id, audio) = upsert_song(&mock, None).await;
        let database_data = select_song(&mock, id).await;
        let database_song: Song = database_data.song.try_into().unwrap();
        let database_property: audio::Property = database_data.property.try_into().unwrap();
        let database_file: file::property::File<_> = database_data.file.into();
        assert_eq!(database_song, audio.metadata.song);
        assert_eq!(database_property, audio.property);
        assert_eq!(database_file, audio.file);
    }

    #[rstest]
    #[tokio::test]
    async fn test_song_update_roundtrip(#[future(awt)] mock: Mock) {
        let (id, _) = upsert_song(&mock, None).await;
        let (id, audio) = upsert_song(&mock, id).await;
        let database_data = select_song(&mock, id).await;
        let database_song: Song = database_data.song.try_into().unwrap();
        let database_property: audio::Property = database_data.property.try_into().unwrap();
        let database_file: file::property::File<_> = database_data.file.into();
        assert_eq!(database_song, audio.metadata.song);
        assert_eq!(database_property, audio.property);
        assert_eq!(database_file, audio.file);
    }
}
