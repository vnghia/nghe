pub mod add;

#[cfg(test)]
pub mod test;

nghe_proc_macro::build_router! {
    modules = [add]
}
