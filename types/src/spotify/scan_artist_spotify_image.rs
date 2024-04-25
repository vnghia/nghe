use nghe_proc_macros::{add_common_convert, add_subsonic_response};
use time::OffsetDateTime;

use crate::time::time_serde;

#[add_common_convert]
pub struct ScanArtistSpotifyImageParams {
    #[serde(
        with = "time_serde::iso8601_datetime_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub artist_updated_at: Option<OffsetDateTime>,
}

#[add_subsonic_response]
pub struct ScanArtistSpotifyImageBody {}

#[cfg(test)]
mod tests {
    use fake::faker::time::en::*;
    use fake::{Fake, Faker};
    use time::macros::datetime;

    use super::*;
    use crate::common::params::{CommonParams, WithCommon};
    #[test]
    fn test_serialize_scan_artist_spotify_image_params() {
        let params = ScanArtistSpotifyImageParams {
            artist_updated_at: DateTimeBetween(
                datetime!(2000-01-01 0:00 UTC),
                datetime!(2100-01-01 0:00 UTC),
            )
            .fake(),
        };
        let params = params.with_common(Faker.fake::<CommonParams>());
        serde_html_form::to_string(params).unwrap();
    }
}
