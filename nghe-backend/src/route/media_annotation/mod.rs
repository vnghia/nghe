mod scrobble;
pub mod star;
pub mod unstar;

nghe_proc_macro::build_router! {
    modules = [scrobble, star, unstar],
}
