pub mod bookmarks;
pub mod browsing;
mod common;
pub mod extension;
pub mod media_list;
pub mod media_retrieval;
pub mod permission;
pub mod scan;
pub mod searching;
pub mod system;
pub mod user;

pub use common::error::{OSError, ServerError};
pub use common::stream::StreamResponse;

#[cfg(test)]
pub mod test {
    pub use super::common::request::CommonParams;
}
