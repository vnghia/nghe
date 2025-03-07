pub mod add;
mod remove;

nghe_proc_macro::build_router! {
    modules = [add(internal = true), remove(internal = true)],
}
