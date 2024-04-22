use std::borrow::Cow;

pub use artists::*;
use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::artists;

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = artists)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct NewArtist<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

pub type ArtistNoId = NewArtist<'static>;

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
