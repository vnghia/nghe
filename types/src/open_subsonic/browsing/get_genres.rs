use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};

use crate::open_subsonic::common::id3::response::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetGenresParams {}

#[add_response_derive]
pub struct Genres {
    pub genre: Vec<GenreId3>,
}

#[add_subsonic_response]
pub struct GenresBody {
    pub genres: Genres,
}
