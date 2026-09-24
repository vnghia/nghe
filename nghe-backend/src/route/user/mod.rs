pub mod create;
mod delete;
mod get;
pub mod list;
mod setup;
mod update;
mod update_password;
mod update_role;

nghe_proc_macro::build_router! {
    modules = [
        create,
        delete,
        get,
        list,
        setup,
        update,
        update_password,
        update_role,
    ],
}
