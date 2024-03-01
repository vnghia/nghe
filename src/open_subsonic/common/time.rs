use time::format_description::well_known::{iso8601, Iso8601};
use time::serde;

const CONFIG: iso8601::EncodedConfig = iso8601::Config::DEFAULT
    .set_year_is_six_digits(false)
    .encode();
const FORMAT: Iso8601<CONFIG> = Iso8601::<CONFIG>;

serde::format_description!(time_format, OffsetDateTime, FORMAT);

pub use time_format::*;
pub mod option {
    pub use super::time_format::option::*;
}
