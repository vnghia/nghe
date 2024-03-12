use crate::OSError;

use anyhow::Result;
use lofty::Accessor;

pub fn extract_common_tags<T: Accessor>(tag: &mut T) -> Result<(String, String)> {
    let title = tag
        .title()
        .ok_or_else(|| OSError::NotFound("Title tag".into()))?
        .into_owned();
    let album = tag
        .album()
        .ok_or_else(|| OSError::NotFound("Album tag".into()))?
        .into_owned();
    Ok((title, album))
}

pub fn parse_number_and_total(
    number_value: Option<&str>,
    total_value: Option<&str>,
) -> Result<(Option<u32>, Option<u32>)> {
    if let Some(number_value) = number_value {
        if let Some((number_value, total_value)) = number_value.split_once('/') {
            // mpeg tag does not have a separate value for total value.
            // therefore if total value is present and number value is not,
            // a negative value will be written to number value.
            Ok((
                if !cfg!(test) {
                    Some(number_value.parse()?)
                } else {
                    number_value.parse().ok()
                },
                Some(total_value.parse()?),
            ))
        } else {
            Ok((
                Some(number_value.parse()?),
                total_value.map(|v| v.parse()).transpose()?,
            ))
        }
    } else {
        Ok((None, total_value.map(|v| v.parse()).transpose()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_and_total() {
        let (number, total) = parse_number_and_total(None, None).unwrap();
        assert!(number.is_none());
        assert!(total.is_none());

        let (number, total) = parse_number_and_total(Some("10"), None).unwrap();
        assert_eq!(number, Some(10));
        assert!(total.is_none());

        let (number, total) = parse_number_and_total(None, Some("20")).unwrap();
        assert!(number.is_none());
        assert_eq!(total, Some(20));

        let (number, total) = parse_number_and_total(Some("10"), Some("20")).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));

        let (number, total) = parse_number_and_total(Some("10/20"), None).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));

        let (number, total) = parse_number_and_total(Some("10/20"), Some("30")).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));
    }
}
