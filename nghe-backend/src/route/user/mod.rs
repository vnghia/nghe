pub mod create;
mod get;
mod list;
mod setup;

nghe_proc_macro::build_router! {
    modules = [
        create(internal = true),
        get(internal = true),
        list(internal = true),
        setup(internal = true),
    ],
}
