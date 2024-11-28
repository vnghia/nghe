mod get_playqueue;
mod save_playqueue;

nghe_proc_macro::build_router! {
    modules = [get_playqueue, save_playqueue],
}
