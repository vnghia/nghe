pub mod download;
mod offset;

nghe_proc_macro::build_router! {
    modules = [download],
    filesystem = true,
}
