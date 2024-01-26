use itertools::Itertools;
use lofty::FileType;

pub const SONG_FILE_TYPES: [FileType; 2] = [FileType::Flac, FileType::Mpeg];

pub const fn to_extension(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Flac => "flac",
        FileType::Mpeg => "mp3",
        _ => "",
    }
}

pub fn to_extensions() -> Vec<&'static str> {
    SONG_FILE_TYPES.iter().map(to_extension).collect_vec()
}
