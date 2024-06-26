pub mod time_serde {
    use time::format_description::well_known::{iso8601, Iso8601};
    use time::serde;

    const ISO8601_CONFIG: iso8601::EncodedConfig =
        iso8601::Config::DEFAULT.set_year_is_six_digits(false).encode();
    const ISO8601_FORMAT: Iso8601<ISO8601_CONFIG> = Iso8601::<ISO8601_CONFIG>;
    serde::format_description!(iso8601_datetime_format, OffsetDateTime, ISO8601_FORMAT);

    pub mod iso8601_datetime {
        pub use super::iso8601_datetime_format::*;
    }

    pub mod iso8601_datetime_option {
        pub use super::iso8601_datetime_format::option::*;
    }
}
