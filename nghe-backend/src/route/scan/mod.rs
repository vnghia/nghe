mod start;

use crate::scan::scanner;

nghe_proc_macro::build_router! {
    modules = [start],
    filesystem = true,
    extensions = [scanner::Config],
}
