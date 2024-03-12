pub mod file_type;
mod parse;

pub use parse::SongInformation;

#[cfg(test)]
pub use parse::test;
