pub mod create_playlist;
pub mod get_playlist;

nghe_proc_macro::build_router! {
    modules = [create_playlist]
}
