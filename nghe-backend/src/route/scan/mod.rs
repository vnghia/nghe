mod start;

use crate::integration::Informant;
use crate::scan::scanner;

nghe_proc_macro::build_router! {
    modules = [start(internal = true)],
    filesystem = true,
    extensions = [scanner::Config, Informant],
}
