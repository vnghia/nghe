#[cfg(test)]
use fake::{Dummy, Fake};
use isolang::Language;
#[cfg(test)]
use itertools::Itertools;
use o2o::o2o;

use super::{Genres, artist, name_date_mbz, position};
use crate::file::image::Image;
use crate::file::lyric::Lyric;
use crate::orm::songs;
use crate::{Error, error};

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
        |language| language.as_str().parse().map_err(error::Kind::from)
    ).try_collect()?)]
    #[into(~.iter().map(|language| language.to_639_3().into()).collect())]
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
    pub album: name_date_mbz::Album<'a>,
    pub artists: artist::Artists<'a>,
    pub genres: Genres<'a>,
    #[cfg_attr(test, dummy(expr = "Lyric::fake_vec()"))]
    pub lyrics: Vec<Lyric<'a>>,
    pub image: Option<Image<'a>>,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::{self, audio};
    use crate::test::{Information, Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_song_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] update_song: bool,
    ) {
        let song: audio::Information = Faker.fake();
        let album_id = song.metadata.album.upsert_mock(&mock, 0).await;
        let id = song
            .upsert_song(mock.database(), album_id.into(), Faker.fake::<String>(), None)
            .await
            .unwrap();

        let database_data = Information::query_data(&mock, id).await;
        let database_song: Song = database_data.song.try_into().unwrap();
        let database_property: audio::Property = database_data.property.try_into().unwrap();
        let database_file: file::Property<_> = database_data.file.try_into().unwrap();
        assert_eq!(database_song, song.metadata.song);
        assert_eq!(database_property, song.property);
        assert_eq!(database_file, song.file);

        if update_song {
            let update_song: audio::Information = Faker.fake();
            let update_id = update_song
                .upsert_song(mock.database(), album_id.into(), Faker.fake::<String>(), id)
                .await
                .unwrap();

            let update_database_data = Information::query_data(&mock, update_id).await;
            let update_database_song: Song = update_database_data.song.try_into().unwrap();
            let update_database_property: audio::Property =
                update_database_data.property.try_into().unwrap();
            let update_database_file: file::Property<_> =
                update_database_data.file.try_into().unwrap();
            assert_eq!(update_database_song, update_song.metadata.song);
            assert_eq!(update_database_property, update_song.property);
            assert_eq!(update_database_file, update_song.file);
        }
    }
}
