use itertools::Itertools;
use lofty::{FileType, MimeType};

pub const SONG_FILE_TYPES: [FileType; 2] = [FileType::Flac, FileType::Mpeg];

pub const fn to_extension(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Flac => "flac",
        FileType::Mpeg => "mp3",
        _ => "bin",
    }
}

pub const fn picture_to_extension(mime_type: &MimeType) -> &'static str {
    match mime_type {
        MimeType::Png => "png",
        MimeType::Jpeg => "jpeg",
        MimeType::Tiff => "tiff",
        MimeType::Bmp => "bmp",
        MimeType::Gif => "gif",
        _ => "bin",
    }
}

pub const fn to_glob_pattern(file_type: &FileType) -> &'static str {
    match file_type {
        FileType::Flac => "**/*.flac",
        FileType::Mpeg => "**/*.mp3",
        _ => "",
    }
}

pub fn to_extensions() -> Vec<&'static str> {
    SONG_FILE_TYPES.iter().map(to_extension).collect_vec()
}
