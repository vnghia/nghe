pub mod create;

nghe_proc_macro::build_router! {
    modules = [create(internal = true)],
}
