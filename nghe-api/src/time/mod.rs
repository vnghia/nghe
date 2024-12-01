#![allow(clippy::ref_option)]

pub mod duration;

pub mod serde {
    use time::format_description::well_known::{iso8601, Iso8601};

    const ISO8601_CONFIG: iso8601::EncodedConfig =
        iso8601::Config::DEFAULT.set_year_is_six_digits(false).encode();
    const ISO8601_FORMAT: Iso8601<ISO8601_CONFIG> = Iso8601::<ISO8601_CONFIG>;
    time::serde::format_description!(iso8601_serde, OffsetDateTime, ISO8601_FORMAT);

    pub use iso8601_serde::{deserialize, serialize};

    pub mod option {
        pub use super::iso8601_serde::option::{deserialize, serialize};
    }
}
