mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const OPEN_SUBSONIC_VERSION: &'static str = "v1.16.1";
pub const SERVER_TYPE: &'static str = "Nghe";
pub const SERVER_VERSION: &'static str = built_info::PKG_VERSION;
