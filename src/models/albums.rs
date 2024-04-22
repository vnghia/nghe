use std::borrow::Cow;

pub use albums::*;
use diesel::prelude::*;
use nghe_proc_macros::generate_date_db;
use uuid::Uuid;

pub use crate::schema::albums;

generate_date_db!(albums);

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = albums)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct NewAlbum<'a> {
    pub name: Cow<'a, str>,
    #[diesel(embed)]
    pub date: AlbumDateDb,
    #[diesel(embed)]
    pub release_date: AlbumReleaseDateDb,
    #[diesel(embed)]
    pub original_release_date: AlbumOriginalReleaseDateDb,
    pub mbz_id: Option<Uuid>,
}

pub type AlbumNoId = NewAlbum<'static>;

#[cfg(test)]
mod test {
    use fake::{Dummy, Fake, Faker};

    use super::*;
    use crate::utils::song::MediaDateMbz;

    impl<'a> Dummy<Faker> for NewAlbum<'a> {
        fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            Faker.fake_with_rng::<MediaDateMbz, _>(rng).into()
        }
    }

    impl<'a> From<&'a str> for NewAlbum<'a> {
        fn from(value: &'a str) -> Self {
            NewAlbum { name: value.into(), ..Default::default() }
        }
    }
}
