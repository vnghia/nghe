fn main() {
    built::write_built_file().expect("Could not acquire build-time information");

    println!("cargo::rustc-check-cfg=cfg(hearing_test)");
    if std::env::var("NGHE_HEARING_TEST_INPUT").is_ok_and(|s| !s.is_empty())
        && std::env::var("NGHE_HEARING_TEST_OUTPUT").is_ok_and(|s| !s.is_empty())
    {
        println!("cargo::rustc-cfg=hearing_test");
    }

    println!("cargo::rustc-check-cfg=cfg(spotify_env)");
    if std::env::var("SPOTIFY_ID").is_ok_and(|s| !s.is_empty())
        && std::env::var("SPOTIFY_SECRET").is_ok_and(|s| !s.is_empty())
    {
        println!("cargo::rustc-cfg=spotify_env");
    }

    println!("cargo::rustc-check-cfg=cfg(lastfm_env)");
    if std::env::var("LASTFM_KEY").is_ok_and(|s| !s.is_empty()) {
        println!("cargo::rustc-cfg=lastfm_env");
    }

    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
