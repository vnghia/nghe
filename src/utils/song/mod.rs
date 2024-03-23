pub mod file_type;
mod parse;
mod transcode;

#[cfg(test)]
pub use parse::test;
pub use parse::SongInformation;
pub use transcode::transcode;
