pub mod add;

nghe_proc_macro::build_router! {
    modules = [add],
    filesystem = true,
}
