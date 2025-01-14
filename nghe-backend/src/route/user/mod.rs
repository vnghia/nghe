pub mod create;
mod get;
mod setup;

nghe_proc_macro::build_router! {
    modules = [
        create(internal = true),
        get(internal = true),
        setup(internal = true),
    ],
}
