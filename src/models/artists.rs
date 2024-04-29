use std::borrow::Cow;

pub use artists::*;
use diesel::prelude::*;
use nghe_types::browsing::get_artist_info2::ArtistInfo;
use nghe_types::id3::InfoId3;
use uuid::Uuid;

use crate::open_subsonic::sql::coalesceid;
use crate::open_subsonic::sql::coalesceid::HelperType as CoalesceId;
pub use crate::schema::artists;

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct NewArtist<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct NewArtistWithIndex<'a> {
    #[diesel(embed)]
    pub new_artist: NewArtist<'a>,
    pub index: Cow<'a, str>,
}

#[derive(Debug, AsChangeset, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
pub struct LastfmInfo<'a> {
    pub lastfm_url: Option<Cow<'a, str>>,
    #[diesel(select_expression = coalesceid(mbz_id, lastfm_mbz_id))]
    #[diesel(select_expression_type = CoalesceId<mbz_id, lastfm_mbz_id>)]
    pub lastfm_mbz_id: Option<Uuid>,
    pub lastfm_biography: Option<Cow<'a, str>>,
}

pub type ArtistNoId = NewArtist<'static>;

impl<'a> From<&'a ArtistNoId> for NewArtist<'a> {
    fn from(value: &'a ArtistNoId) -> Self {
        NewArtist { name: AsRef::<str>::as_ref(&value.name).into(), mbz_id: value.mbz_id }
    }
}

impl From<(String, Option<Uuid>)> for ArtistNoId {
    fn from(value: (String, Option<Uuid>)) -> Self {
        Self { name: value.0.into(), mbz_id: value.1 }
    }
}

impl From<(&str, Option<Uuid>)> for ArtistNoId {
    fn from(value: (&str, Option<Uuid>)) -> Self {
        (value.0.to_string(), value.1).into()
    }
}

impl From<LastfmInfo<'static>> for ArtistInfo {
    fn from(value: LastfmInfo<'static>) -> Self {
        Self {
            biography: value.lastfm_biography.map(Cow::into_owned),
            info: InfoId3 {
                music_brainz_id: value.lastfm_mbz_id,
                last_fm_url: value.lastfm_url.map(Cow::into_owned),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::RangeBounds;

    use fake::{Dummy, Fake, Faker};

    use super::*;

    impl Dummy<Faker> for ArtistNoId {
        fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            Self {
                name: Faker.fake_with_rng::<String, _>(rng).into(),
                mbz_id: Faker.fake_with_rng(rng),
            }
        }
    }

    impl From<String> for ArtistNoId {
        fn from(value: String) -> Self {
            Self { name: value.into(), mbz_id: None }
        }
    }

    impl From<&str> for ArtistNoId {
        fn from(value: &str) -> Self {
            value.to_string().into()
        }
    }

    impl From<ArtistNoId> for (String, String) {
        fn from(value: ArtistNoId) -> Self {
            let artist_name = value.name.into_owned();
            let artist_mbz_id = if let Some(artist_mbz_id) = value.mbz_id {
                artist_mbz_id.to_string()
            } else {
                String::default()
            };
            (artist_name, artist_mbz_id)
        }
    }

    impl ArtistNoId {
        pub fn fake_vec<R>(range: R) -> Vec<ArtistNoId>
        where
            R: RangeBounds<usize>,
            usize: fake::Dummy<R>,
        {
            // Flac does not keep the order of these tags. So we only generate one mbz id per file
            // to simplify testing.
            let size = range.fake();
            if size == 1 {
                fake::vec![ArtistNoId; 1]
            } else {
                (0..size).map(|_| ArtistNoId { mbz_id: None, ..Faker.fake() }).collect()
            }
        }
    }
}
