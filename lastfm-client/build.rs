fn main() {
    if std::env::var("LASTFM_KEY").is_ok() {
        println!("cargo::rustc-cfg=lastfm_env")
    }
}
