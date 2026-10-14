use std::marker::ConstParamTy;

use nghe_proc_macro::api_derive;
use strum::EnumString;

#[repr(i16)]
#[api_derive(fake = true)]
#[derive(Clone, Copy, PartialEq, Eq, ConstParamTy, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Type {
    Local,
    S3,
}
