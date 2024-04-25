fn main() {
    if std::env::var("LASTFM_KEY").is_ok_and(|s| !s.is_empty()) {
        println!("cargo::rustc-cfg=lastfm_env")
    }
}
