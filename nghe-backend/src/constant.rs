mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use built_info::PKG_NAME;
