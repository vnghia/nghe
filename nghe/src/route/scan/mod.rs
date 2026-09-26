mod start;

use crate::integration::Informant;
use crate::scan::scanner;

nghe_proc_macro::build_router! {
    modules = [start],
    filesystem = true,
    extensions = [scanner::Config, Informant],
}
