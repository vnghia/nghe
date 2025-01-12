pub mod create;
mod info;
mod setup;

nghe_proc_macro::build_router! {
    modules = [
        create(internal = true),
        info(internal = true),
        setup(internal = true),
    ],
}
