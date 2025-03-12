pub mod add;
mod remove;
pub mod update;

nghe_proc_macro::build_router! {
    modules = [
        add(internal = true),
        remove(internal = true),
        update(internal = true),
    ],
}
