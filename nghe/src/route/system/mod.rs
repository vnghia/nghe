mod get_open_subsonic_extensions;
mod health;
mod ping;

nghe_proc_macro::build_router! {
    modules = [get_open_subsonic_extensions, health, ping],
}
