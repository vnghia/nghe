mod scrobble;

pub fn router() -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(scrobble)
}

#[cfg(test)]
pub mod test {
    pub use super::scrobble::scrobble;
}
