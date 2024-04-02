pub mod file_type;
mod parse;
mod transcode;

#[cfg(test)]
pub use parse::test;
pub use parse::SongInformation;
#[cfg(test)]
pub use transcode::test::transcode_to_memory;
pub use transcode::transcode;
