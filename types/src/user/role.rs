use std::marker::ConstParamTy;

use nghe_proc_macros::add_types_derive;

#[add_types_derive]
#[derive(Debug, ConstParamTy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Role {
    pub admin_role: bool,
    pub stream_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}

#[cfg(feature = "test")]
impl Role {
    pub const fn const_default() -> Self {
        Self { admin_role: false, stream_role: false, download_role: false, share_role: false }
    }
}

#[cfg(feature = "test")]
impl Default for Role {
    fn default() -> Self {
        Self::const_default()
    }
}
