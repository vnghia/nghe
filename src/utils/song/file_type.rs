use lofty::FileType;

pub const SONG_FILE_TYPES: [FileType; 2] = [FileType::Flac, FileType::Mpeg];

#[cfg(test)]
pub mod tests {
    use super::*;

    use itertools::Itertools;

    pub fn to_extension(file_type: &FileType) -> &'static str {
        match file_type {
            FileType::Flac => "flac",
            FileType::Mpeg => "mp3",
            _ => unimplemented!("file type not supported"),
        }
    }

    pub fn to_extensions() -> Vec<&'static str> {
        SONG_FILE_TYPES.iter().map(to_extension).collect_vec()
    }
}
