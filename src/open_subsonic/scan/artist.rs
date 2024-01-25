use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

pub async fn upsert_artists<TI: AsRef<str>, TN: AsRef<str>>(
    pool: &DatabasePool,
    ignored_prefixes: &[TI],
    names: &[TN],
) -> OSResult<Vec<artists::Artist>> {
    Ok(diesel::insert_into(artists::table)
        .values(
            names
                .iter()
                .map(|name| artists::NewArtist {
                    name: std::borrow::Cow::Borrowed(name.as_ref()),
                    index: build_artist_index(ignored_prefixes, name.as_ref()).into(),
                })
                .collect_vec(),
        )
        .on_conflict(artists::name)
        .do_update()
        .set(artists::scanned_at.eq(time::OffsetDateTime::now_utc()))
        .returning(artists::Artist::as_returning())
        .get_results(&mut pool.get().await?)
        .await?)
}

// TODO: better index building mechanism
pub fn build_artist_index<T: AsRef<str>>(ignored_prefixes: &[T], name: &str) -> String {
    for ignored_prefix in ignored_prefixes {
        if let Some(stripped) = name.strip_prefix(ignored_prefix.as_ref()) {
            if let Some(index_char) = stripped.chars().next() {
                return index_char_to_string(index_char);
            }
        }
    }

    if let Some(index_char) = name.chars().next() {
        index_char_to_string(index_char)
    } else {
        unreachable!("name can not be empty")
    }
}

fn index_char_to_string(index_char: char) -> String {
    if index_char.is_ascii_alphabetic() {
        index_char.to_ascii_uppercase().to_string()
    } else if index_char.is_numeric() {
        "#".to_owned()
    } else {
        "*".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_char_to_string_numeric() {
        assert_eq!(index_char_to_string('1'), "#");
    }

    #[test]
    fn test_index_char_to_string_alphabetic_upper() {
        assert_eq!(index_char_to_string('A'), "A");
    }

    #[test]
    fn test_index_char_to_string_alphabetic_lower() {
        assert_eq!(index_char_to_string('a'), "A");
    }

    #[test]
    fn test_index_char_to_string_non_alphabetic() {
        assert_eq!(index_char_to_string('%'), "*");
    }

    #[test]
    fn test_index_char_to_string_non_ascii() {
        assert_eq!(index_char_to_string('é'), "*");
    }

    #[test]
    fn test_build_artist_index_with_article() {
        assert_eq!(build_artist_index(&["The ", "A "], "The test"), "T");
    }

    #[test]
    fn test_build_artist_index_no_article() {
        assert_eq!(build_artist_index(&["The ", "A "], "test"), "T");
    }
}
