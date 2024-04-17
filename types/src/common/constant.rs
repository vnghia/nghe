mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const fn git_commit_hash_short() -> &'static str {
    if built_info::GIT_COMMIT_HASH_SHORT.is_some() {
        built_info::GIT_COMMIT_HASH_SHORT.unwrap()
    } else {
        "0"
    }
}

pub const OPEN_SUBSONIC_VERSION: &str = "1.16.1";
pub const SERVER_NAME: &str = "nghe";
pub const SERVER_VERSION: &str =
    constcat::concat!(built_info::PKG_VERSION, " (", git_commit_hash_short(), ")");
