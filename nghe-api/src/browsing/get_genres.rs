use nghe_proc_macro::api_derive;

use crate::id3;

#[api_derive]
#[endpoint(path = "getGenres")]
pub struct Request {}

#[api_derive(response = true)]
pub struct Genres {
    pub genre: Vec<id3::genre::WithCount>,
}

#[api_derive]
pub struct Response {
    pub genres: Genres,
}
