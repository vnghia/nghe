use std::borrow::Cow;

#[cfg(test)]
use fake::{Dummy, Fake};
use isolang::Language;
#[cfg(test)]
use itertools::Itertools;

use super::{artist, name_date_mbz, position};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Song<'a> {
    pub main: name_date_mbz::NameDateMbz<'a>,
    pub track_disc: position::TrackDisc,
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
