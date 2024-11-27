use o2o::o2o;

use crate::orm::songs;
use crate::Error;

#[derive(Debug, Default, Clone, Copy, o2o)]
#[try_map_owned(songs::position::Track, Error)]
#[try_map_owned(songs::position::Disc, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct Position {
    #[from(~.map(u16::try_from).transpose()?)]
    #[into(~.map(u16::into))]
    pub number: Option<u16>,
    #[from(~.map(u16::try_from).transpose()?)]
    #[into(~.map(u16::into))]
    pub total: Option<u16>,
}

#[derive(Debug, Default, Clone, Copy, o2o)]
#[try_map_owned(songs::position::TrackDisc, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy))]
pub struct TrackDisc {
    #[map(~.try_into()?)]
    pub track: Position,
    #[map(~.try_into()?)]
    pub disc: Position,
}

impl Position {
    fn parse(number_str: Option<&str>, total_str: Option<&str>) -> Option<Self> {
        if let Some(number) = number_str {
            // Prioritize parsing from number_str if there is a slash inside
            if let Some((number, total)) = number.split_once('/') {
                let number = Some(number.parse().ok()?);
                let total = if total.is_empty() {
                    total_str.map(str::parse).transpose().ok()?
                } else {
                    Some(total.parse().ok()?)
                };
                Some(Self { number, total })
            } else {
                let number = Some(number.parse().ok()?);
                let total = total_str.map(str::parse).transpose().ok()?;
                Some(Self { number, total })
            }
        } else {
            let total = total_str.map(str::parse).transpose().ok()?;
            Some(Self { number: None, total })
        }
    }
}

impl TrackDisc {
    pub fn parse(
        track_number: Option<&str>,
        track_total: Option<&str>,
        disc_number: Option<&str>,
        disc_total: Option<&str>,
    ) -> Result<Self, Error> {
        if let Some(track) = Position::parse(track_number, track_total)
            && let Some(disc) = Position::parse(disc_number, disc_total)
        {
            Ok(Self { track, disc })
        } else if let Some(track_disc) = Self::parse_vinyl_position(track_number) {
            Ok(track_disc)
        } else {
            Err(Error::MediaPositionFormat {
                track_number: track_number.map(str::to_owned),
                track_total: track_total.map(str::to_owned),
                disc_number: disc_number.map(str::to_owned),
                disc_total: disc_total.map(str::to_owned),
            })
        }
    }

    // This position format is encountered when extracting metadata from some Vinyl records.
    fn parse_vinyl_position(str: Option<&str>) -> Option<Self> {
        if let Some(str) = str
            && let Some(disc_letter) = str.chars().next()
            && disc_letter.is_ascii_alphabetic()
        {
            // In ASCII, `A` is equal to 65.
            let disc_number = (disc_letter.to_ascii_uppercase() as u8 - 64).into();
            let track_number = str[1..].parse().ok()?;
            Some(Self {
                track: Position { number: Some(track_number), total: None },
                disc: Position { number: Some(disc_number), total: None },
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_arguments)]

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(None, None, None, None)]
    #[case(Some("10"), None, Some(10), None)]
    #[case(None, Some("20"), None, Some(20))]
    #[case(Some("10"), Some("20"), Some(10), Some(20))]
    #[case(Some("10/20"), None, Some(10), Some(20))]
    #[case(Some("10/20"), Some("30"), Some(10), Some(20))]
    #[case(Some("10/"), None, Some(10), None)]
    #[case(Some("10/"), Some("30"), Some(10), Some(30))]
    fn test_parse_position_success(
        #[case] number_str: Option<&str>,
        #[case] total_str: Option<&str>,
        #[case] number: Option<u16>,
        #[case] total: Option<u16>,
    ) {
        let position = Position::parse(number_str, total_str).unwrap();
        assert_eq!(position.number, number);
        assert_eq!(position.total, total);
    }

    #[rstest]
    #[case(Some("/"), None)]
    #[case(Some("-10/20"), None)]
    #[case(Some("-10"), Some("-20"))]
    #[case(None, Some("A"))]
    #[case(None, Some("10/20"))]
    fn test_parse_position_error(
        #[case] number_str: Option<&str>,
        #[case] total_str: Option<&str>,
    ) {
        assert!(Position::parse(number_str, total_str).is_none());
    }

    #[rstest]
    #[case(Some("A1"), 1, 1)]
    #[case(Some("E100"), 100, 5)]
    #[case(Some("c1000"), 1000, 3)]
    #[case(Some("Z0"), 0, 26)]
    fn test_parse_vinyl_position_success(
        #[case] str: Option<&str>,
        #[case] track_number: u16,
        #[case] disc_number: u16,
    ) {
        let track_disc = TrackDisc::parse_vinyl_position(str).unwrap();
        assert_eq!(track_disc.track.number, Some(track_number));
        assert!(track_disc.track.total.is_none());
        assert_eq!(track_disc.disc.number, Some(disc_number));
        assert!(track_disc.disc.total.is_none());
    }

    #[rstest]
    #[case(None)]
    #[case(Some("1A"))]
    #[case(Some("A1B"))]
    #[case(Some("1000"))]
    fn test_parse_vinyl_position_error(#[case] str: Option<&str>) {
        assert!(TrackDisc::parse_vinyl_position(str).is_none());
    }

    #[rstest]
    #[case(None, None, None, None, None, None, None, None)]
    #[case(Some("A2"), None, None, None, Some(2), None, Some(1), None)]
    #[case(Some("1/"), None, None, Some("10"), Some(1), None, None, Some(10))]
    #[case(Some("10"), Some("20"), Some("2/5"), None, Some(10), Some(20), Some(2), Some(5))]
    fn test_parse_track_disc_success(
        #[case] track_number_str: Option<&str>,
        #[case] track_total_str: Option<&str>,
        #[case] disc_number_str: Option<&str>,
        #[case] disc_total_str: Option<&str>,
        #[case] track_number: Option<u16>,
        #[case] track_total: Option<u16>,
        #[case] disc_number: Option<u16>,
        #[case] disc_total: Option<u16>,
    ) {
        let track_disc =
            TrackDisc::parse(track_number_str, track_total_str, disc_number_str, disc_total_str)
                .unwrap();
        assert_eq!(track_disc.track.number, track_number);
        assert_eq!(track_disc.track.total, track_total);
        assert_eq!(track_disc.disc.number, disc_number);
        assert_eq!(track_disc.disc.total, disc_total);
    }

    #[rstest]
    #[case(Some("1A"), None, None, None)]
    #[case(Some("10"), Some("B"), None, None)]
    #[case(Some("10"), None, Some("20/Z"), None)]
    fn test_parse_track_disc_error(
        #[case] track_number_str: Option<&str>,
        #[case] track_total_str: Option<&str>,
        #[case] disc_number_str: Option<&str>,
        #[case] disc_total_str: Option<&str>,
    ) {
        assert!(
            TrackDisc::parse(track_number_str, track_total_str, disc_number_str, disc_total_str)
                .is_err()
        );
    }
}
