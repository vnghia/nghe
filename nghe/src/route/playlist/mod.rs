pub mod create;
mod delete;
pub mod get;
mod list;
mod update;

nghe_proc_macro::build_router! {
    modules = [create, delete, get, list, update]
}
