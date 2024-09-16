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
#[try_map_owned(songs::Data<'a>, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Song<'a> {
    #[map(~.try_into()?)]
    pub main: name_date_mbz::NameDateMbz<'a>,
    #[map(~.try_into()?)]
    pub track_disc: position::TrackDisc,
    #[from(~.into_iter().map(|language| Language::from_str(language.as_ref())).try_collect()?)]
    #[into(~.into_iter().map(|language| language.to_639_3().into()).collect())]
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
    use crate::orm::albums;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_album_roundtrip(#[future(awt)] mock: Mock) {
        let album: name_date_mbz::NameDateMbz = Faker.fake();
        let id: Uuid = diesel::insert_into(albums::table)
            .values(albums::Upsert {
                music_folder_id: mock.music_folder(0).await.music_folder.id,
                data: album.clone().try_into().unwrap(),
            })
            .returning(albums::schema::id)
            .get_result(&mut mock.get().await)
            .await
            .unwrap();
        let database_album: name_date_mbz::NameDateMbz = albums::table
            .filter(albums::schema::id.eq(id))
            .select(albums::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(database_album, album);
    }

    #[rstest]
    #[tokio::test]
    async fn test_album_update_roundtrip(#[future(awt)] mock: Mock) {
        let album: name_date_mbz::NameDateMbz = Faker.fake();
        let id: Uuid = diesel::insert_into(albums::table)
            .values(albums::Upsert {
                music_folder_id: mock.music_folder(0).await.music_folder.id,
                data: album.try_into().unwrap(),
            })
            .returning(albums::schema::id)
            .get_result(&mut mock.get().await)
            .await
            .unwrap();
        let album: name_date_mbz::NameDateMbz = Faker.fake();
        diesel::update(albums::table)
            .set(albums::Upsert {
                music_folder_id: mock.music_folder(0).await.music_folder.id,
                data: album.clone().try_into().unwrap(),
            })
            .execute(&mut mock.get().await)
            .await
            .unwrap();
        let database_album: name_date_mbz::NameDateMbz = albums::table
            .filter(albums::schema::id.eq(id))
            .select(albums::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(database_album, album);
    }
}
