use anyhow::Result;
use concat_string::concat_string;
use itertools::Itertools;
use tracing::instrument;
use uuid::Uuid;

use crate::models::*;
use crate::OSError;

fn parse_number_and_total(
    number_value: Option<&str>,
    total_value: Option<&str>,
) -> Result<(Option<u32>, Option<u32>)> {
    if let Some(number_value) = number_value {
        if let Some((number_value, total_value)) = number_value.split_once('/') {
            // mpeg tag does not have a separate value for total value.
            // therefore if total value is present and number value is not,
            // a negative value will be written to number value.
            Ok((
                if !cfg!(test) { Some(number_value.parse()?) } else { number_value.parse().ok() },
                Some(total_value.parse()?),
            ))
        } else {
            Ok((Some(number_value.parse()?), total_value.map(|v| v.parse()).transpose()?))
        }
    } else {
        Ok((None, total_value.map(|v| v.parse()).transpose()?))
    }
}

fn parse_track_and_disc_number_letter_prefix(track_number_value: &str) -> Result<(u32, u32)> {
    if let Some(disc_letter) = track_number_value.chars().next()
        && disc_letter.is_ascii_alphabetic()
    {
        // 'A' = 65 ASCII
        let disc_number = (disc_letter.to_ascii_uppercase() as u8 - 64) as _;
        let track_number = track_number_value[1..].parse()?;
        Ok((track_number, disc_number))
    } else {
        anyhow::bail!(OSError::InvalidParameter(
            concat_string!("track number ", track_number_value).into()
        ))
    }
}

pub type NumberTotal = (Option<u32>, Option<u32>);
#[instrument(err(Debug))]
pub fn parse_track_and_disc(
    track_number: Option<&str>,
    track_total: Option<&str>,
    disc_number: Option<&str>,
    disc_total: Option<&str>,
) -> Result<(NumberTotal, NumberTotal)> {
    match try {
        let track_result = parse_number_and_total(track_number, track_total)?;
        let disc_result = parse_number_and_total(disc_number, disc_total)?;
        (track_result, disc_result)
    } {
        Err::<_, anyhow::Error>(e) if let Some(v) = track_number => {
            match parse_track_and_disc_number_letter_prefix(v) {
                Ok(r) => Ok(((Some(r.0), None), (Some(r.1), None))),
                Err(le) => {
                    anyhow::bail!(concat_string!(e.to_string(), "; ", le.to_string()))
                }
            }
        }
        r => r,
    }
}

pub fn to_artist_no_ids(
    artist_names: Vec<String>,
    artist_mbz_ids: Option<Vec<&str>>,
) -> Result<Vec<artists::ArtistNoId>> {
    let artist_mbz_ids = if let Some(artist_mbz_ids) = artist_mbz_ids {
        if artist_names.len() < artist_mbz_ids.len() {
            anyhow::bail!(OSError::InvalidParameter(
                "mbz ids must have small or equal size to names".into()
            ));
        } else {
            artist_mbz_ids
                .into_iter()
                .map(|v| if v.is_empty() { Ok(None) } else { Uuid::parse_str(v).map(Some) })
                .try_collect()?
        }
    } else {
        vec![]
    };
    Ok(artist_names
        .into_iter()
        .zip(artist_mbz_ids.into_iter().chain(std::iter::repeat(None)))
        .map(|v| v.into())
        .collect_vec())
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

    #[test]
    fn test_parse_track_and_disc_number_letter_prefix() {
        let (track, disc) = parse_track_and_disc_number_letter_prefix("A1").unwrap();
        assert_eq!(track, 1);
        assert_eq!(disc, 1);

        let (track, disc) = parse_track_and_disc_number_letter_prefix("E100").unwrap();
        assert_eq!(track, 100);
        assert_eq!(disc, 5);

        let (track, disc) = parse_track_and_disc_number_letter_prefix("c1000").unwrap();
        assert_eq!(track, 1000);
        assert_eq!(disc, 3);

        assert!(parse_track_and_disc_number_letter_prefix("ab12").is_err());
        assert!(parse_track_and_disc_number_letter_prefix("ab12c").is_err());
        assert!(parse_track_and_disc_number_letter_prefix("1000").is_err());
    }

    #[test]
    fn test_parse_track_and_disc() {
        let ((track_number, track_total), (disc_number, disc_total)) =
            parse_track_and_disc(None, None, None, None).unwrap();
        assert!(track_number.is_none());
        assert!(track_total.is_none());
        assert!(disc_number.is_none());
        assert!(disc_total.is_none());

        let ((track_number, track_total), (disc_number, disc_total)) =
            parse_track_and_disc(Some("10/20"), None, None, Some("30")).unwrap();
        assert_eq!(track_number, Some(10));
        assert_eq!(track_total, Some(20));
        assert!(disc_number.is_none());
        assert_eq!(disc_total, Some(30));

        let ((track_number, track_total), (disc_number, disc_total)) =
            parse_track_and_disc(Some("B10"), None, None, Some("30")).unwrap();
        assert_eq!(track_number, Some(10));
        assert!(track_total.is_none());
        assert_eq!(disc_number, Some(2));
        assert!(disc_total.is_none());

        assert!(parse_track_and_disc(Some("10"), None, Some("A"), Some("30")).is_err());
        assert!(parse_track_and_disc(Some("ab10"), None, None, None).is_err());
        assert!(parse_track_and_disc(None, None, Some("A"), Some("30")).is_err());
    }
}
