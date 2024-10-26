pub mod download;

nghe_proc_macro::build_router! {
    modules = [download],
    filesystem = true,
}
