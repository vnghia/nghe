use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetGenresParams {}

#[add_types_derive]
pub struct Genres {
    pub genre: Vec<GenreId3>,
}

#[add_subsonic_response]
pub struct GenresBody {
    pub genres: Genres,
}

add_request_types_test!(GetGenresParams);
