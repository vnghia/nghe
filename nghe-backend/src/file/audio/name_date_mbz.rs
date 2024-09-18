use std::borrow::Cow;

#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use o2o::o2o;
use uuid::Uuid;

use super::date::Date;
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
