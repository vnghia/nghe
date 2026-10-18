#![allow(clippy::ref_option)]

pub mod signed_duration;

use time::format_description::well_known::{Iso8601, iso8601};

const ISO8601_CONFIG: iso8601::EncodedConfig =
    iso8601::Config::DEFAULT.set_year_is_six_digits(false).encode();
const ISO8601_FORMAT: Iso8601<ISO8601_CONFIG> = Iso8601::<ISO8601_CONFIG>;

pub mod serde {
    use super::ISO8601_FORMAT;

    time::serde::format_description!(iso8601_serde, OffsetDateTime, ISO8601_FORMAT);

    pub use iso8601_serde::{deserialize, serialize};

    pub mod option {
        pub use super::iso8601_serde::option::{deserialize, serialize};
    }
}

pub mod parsing {
    use super::ISO8601_FORMAT;

    pub mod offset_date_time {
        use time::OffsetDateTime;

        use super::ISO8601_FORMAT;

        pub fn from_str(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
            OffsetDateTime::parse(value, &ISO8601_FORMAT)
        }
    }
}
