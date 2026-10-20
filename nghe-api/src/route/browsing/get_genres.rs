use nghe_proc_macro::api_derive;

use crate::id3;

#[api_derive]
#[endpoint(path = "getGenres")]
pub struct Request;

#[api_derive]
pub struct Genres {
    pub genre: Vec<id3::genre::WithCount>,
}

#[api_derive]
pub struct Response {
    pub genres: Genres,
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
