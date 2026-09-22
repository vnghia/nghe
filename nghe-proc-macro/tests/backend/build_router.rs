nghe_proc_macro::build_router! {
    modules = [create(internal = true), read, update, delete(internal = true)],
    filesystem = true,
    extensions = [Extension, config::Network],
}
