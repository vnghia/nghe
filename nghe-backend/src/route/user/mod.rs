pub mod create;
mod delete;
mod get;
pub mod list;
mod setup;

nghe_proc_macro::build_router! {
    modules = [
        create(internal = true),
        delete(internal = true),
        get(internal = true),
        list(internal = true),
        setup(internal = true),
    ],
}
