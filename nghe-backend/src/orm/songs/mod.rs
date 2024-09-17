use diesel::prelude::*;
use diesel_derives::AsChangeset;

pub mod date;
pub mod name_date_mbz;
pub mod position;

use std::borrow::Cow;

pub use crate::schema::songs::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    #[diesel(embed)]
    pub main: name_date_mbz::NameDateMbz<'a>,
    #[diesel(embed)]
    pub track_disc: position::TrackDisc,
    pub languages: Vec<Cow<'a, str>>,
}
