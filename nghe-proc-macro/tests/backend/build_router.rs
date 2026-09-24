nghe_proc_macro::build_router! {
    modules = [create, read, update, delete],
    filesystem = true,
    extensions = [Extension, config::Network],
}
