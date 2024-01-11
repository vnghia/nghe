mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const OPEN_SUBSONIC_VERSION: &str = "v1.16.1";
pub const SERVER_TYPE: &str = "Nghe";
pub const SERVER_VERSION: &str = built_info::PKG_VERSION;
