pub mod add;

#[cfg(test)]
mod test;

nghe_proc_macro::build_router! {
    modules = [add]
}
