mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const OPEN_SUBSONIC_VERSION: &str = "1.16.1";
pub const SERVER_NAME: &str = "nghe";
pub const SERVER_VERSION: &str = const_format::concatc!(
    built_info::PKG_VERSION,
    " (",
    built_info::GIT_COMMIT_HASH_SHORT.unwrap(),
    ")"
);
