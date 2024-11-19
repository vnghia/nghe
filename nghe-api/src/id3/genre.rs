use nghe_proc_macro::api_derive;

#[api_derive(response = true)]
pub struct Genre {
    pub name: String,
}

#[api_derive(response = true)]
#[derive(Default)]
#[serde(transparent)]
pub struct Genres {
    pub value: Vec<Genre>,
}

impl<S: Into<String>> From<S> for Genre {
    fn from(genre: S) -> Self {
        Self { name: genre.into() }
    }
}

impl<S: Into<String>> FromIterator<S> for Genres {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self { value: iter.into_iter().map(Genre::from).collect() }
    }
}
