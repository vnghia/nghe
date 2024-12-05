use std::marker::ConstParamTy;

use nghe_proc_macro::api_derive;

#[repr(i16)]
#[api_derive(fake = true)]
#[derive(Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum Type {
    Local,
    S3,
}
