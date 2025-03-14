pub mod add;
pub mod get;

nghe_proc_macro::build_router! {
    modules = [add(internal = true), get(internal = true)],
    filesystem = true,
}
