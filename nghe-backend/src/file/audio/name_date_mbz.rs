use std::borrow::Cow;

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use o2o::o2o;
use uuid::Uuid;

use super::date::Date;
use crate::database::Database;
use crate::orm::upsert::Insert as _;
use crate::orm::{albums, songs};
use crate::Error;

#[derive(Debug, o2o)]
#[try_map_owned(songs::name_date_mbz::NameDateMbz<'a>, Error)]
#[try_map_owned(albums::Data<'a>, Error)]
#[ref_try_into(songs::name_date_mbz::NameDateMbz<'a>, Error)]
#[ref_try_into(albums::Data<'a>, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct NameDateMbz<'a> {
    #[ref_into(Cow::Borrowed(~.as_ref()))]
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub name: Cow<'a, str>,
    #[map(~.try_into()?)]
    pub date: Date,
    #[map(~.try_into()?)]
    pub release_date: Date,
    #[map(~.try_into()?)]
    pub original_release_date: Date,
    pub mbz_id: Option<Uuid>,
}

pub type Album<'a> = NameDateMbz<'a>;

impl<'a> Album<'a> {
    pub async fn upsert(&self, database: &Database, music_folder_id: Uuid) -> Result<Uuid, Error> {
        albums::Upsert { music_folder_id, data: self.try_into()? }.insert(database).await
    }

    pub async fn cleanup(database: &Database) -> Result<(), Error> {
        // Delete all albums which do not have any song associated.
        let alias_albums = diesel::alias!(albums as alias);
        diesel::delete(albums::table)
            .filter(
                albums::id.eq_any(
                    alias_albums
                        .filter(not(exists(
                            songs::table.filter(songs::album_id.eq(alias_albums.field(albums::id))),
                        )))
                        .select(alias_albums.field(albums::id)),
                ),
            )
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use futures_lite::{stream, StreamExt};

    use super::*;
    use crate::test::Mock;

    impl<'a> Album<'a> {
        pub async fn upsert_mock(&self, mock: &Mock, index: usize) -> Uuid {
            self.upsert(mock.database(), mock.music_folder(index).await.id()).await.unwrap()
        }
    }

    impl Album<'static> {
        pub async fn query(mock: &Mock, id: Uuid) -> Self {
            albums::table
                .filter(albums::id.eq(id))
                .select(albums::Data::as_select())
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
                .try_into()
                .unwrap()
        }

        pub async fn queries(mock: &Mock) -> Vec<Self> {
            let ids = albums::table
                .select(albums::id)
                .order_by(albums::name)
                .get_results(&mut mock.get().await)
                .await
                .unwrap();
            stream::iter(ids).then(async |id| Self::query(mock, id).await).collect().await
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_album_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] mbz_id: bool,
        #[values(true, false)] update_album: bool,
    ) {
        let mbz_id = if mbz_id { Some(Faker.fake()) } else { None };
        let album = Album { mbz_id, ..Faker.fake() };
        let id = album.upsert_mock(&mock, 0).await;
        let database_album = Album::query(&mock, id).await;
        assert_eq!(database_album, album);

        if update_album {
            let update_album = Album { mbz_id, ..Faker.fake() };
            let update_id = update_album.upsert_mock(&mock, 0).await;
            let database_update_album = Album::query(&mock, id).await;
            if mbz_id.is_some() {
                assert_eq!(id, update_id);
                assert_eq!(database_update_album, update_album);
            } else {
                // This will always insert a new row to the database
                // since there is nothing to identify an old album.
                assert_ne!(id, update_id);
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_album_upsert_no_mbz_id(#[future(awt)] mock: Mock) {
        // We want to make sure that insert the same album with no mbz_id
        // twice does not result in any error.
        let album = Album { mbz_id: None, ..Faker.fake() };
        let id = album.upsert_mock(&mock, 0).await;
        let update_id = album.upsert_mock(&mock, 0).await;
        assert_eq!(update_id, id);
    }

    #[rstest]
    #[tokio::test]
    async fn test_combine_album_artist(
        #[future(awt)] mock: Mock,
        #[values(true, false)] compilation: bool,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        let album: Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        let artists: Vec<_> = fake::vec![audio::Artist; 4].into_iter().sorted().collect();
        music_folder
            .add_audio()
            .album(album.clone())
            .artists(audio::Artists {
                song: [artists[0].clone()].into(),
                album: [artists[2].clone()].into(),
                compilation,
            })
            .call()
            .await
            .add_audio()
            .album(album.clone())
            .artists(audio::Artists {
                song: [artists[1].clone()].into(),
                album: [artists[3].clone()].into(),
                compilation,
            })
            .call()
            .await;
        let range = if compilation { 0..4 } else { 2..4 };
        assert_eq!(artists[range], audio::Artist::query_album(&mock, album_id).await);
    }

    mod cleanup {
        use super::*;

        #[rstest]
        #[case(1, 0)]
        #[case(1, 1)]
        #[case(5, 3)]
        #[case(5, 5)]
        #[tokio::test]
        async fn test_album(
            #[future(awt)] mock: Mock,
            #[case] n_song: usize,
            #[case] n_subset: usize,
        ) {
            let mut music_folder = mock.music_folder(0).await;
            let album: Album = Faker.fake();
            music_folder.add_audio().album(album.clone()).n_song(n_song).call().await;
            let song_ids: Vec<_> = music_folder.database.keys().collect();
            assert!(Album::queries(&mock).await.contains(&album));

            diesel::delete(songs::table)
                .filter(songs::id.eq_any(&song_ids[0..n_subset]))
                .execute(&mut mock.get().await)
                .await
                .unwrap();
            Album::cleanup(mock.database()).await.unwrap();
            assert_eq!(Album::queries(&mock).await.contains(&album), n_subset < n_song);
        }
    }
}
