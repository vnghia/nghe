pub mod create;
mod delete;
mod get;
pub mod list;
mod setup;
mod update;
mod update_password;

nghe_proc_macro::build_router! {
    modules = [
        create(internal = true),
        delete(internal = true),
        get(internal = true),
        list(internal = true),
        setup(internal = true),
        update(internal = true),
        update_password(internal = true),
    ],
}
