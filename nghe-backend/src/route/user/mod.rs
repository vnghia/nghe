pub mod create;
mod setup;

nghe_proc_macro::build_router! {
    modules = [
        create(internal = true),
        setup(internal = true),
    ],
}
