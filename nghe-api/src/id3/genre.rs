use nghe_proc_macro::api_derive;

#[api_derive(response = true)]
#[cfg_attr(feature = "test", derive(Clone, Hash))]
pub struct Genre {
    pub name: String,
}

#[api_derive(response = true)]
#[derive(Default)]
#[serde(transparent)]
pub struct Genres {
    pub value: Vec<Genre>,
}

impl Genres {
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
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
