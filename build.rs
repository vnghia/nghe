fn main() {
    println!("cargo::rustc-check-cfg=cfg(lastfm_env)");
    if std::env::var("LASTFM_KEY").is_ok_and(|s| !s.is_empty()) {
        println!("cargo::rustc-cfg=lastfm_env");
    }

    println!("cargo::rustc-check-cfg=cfg(spotify_env)");
    if std::env::var("SPOTIFY_ID").is_ok_and(|s| !s.is_empty()) {
        println!("cargo::rustc-cfg=spotify_env");
    }
}
