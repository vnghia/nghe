use lofty::file::FileType;
use lofty::picture::MimeType;
use phf::phf_map;

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

pub static FILETYPE_GLOB_PATTERN: phf::Map<&'static str, FileType> = phf_map! {
    "**/*.flac" => FileType::Flac,
    "**/*.mp3" => FileType::Mpeg,
};

pub static SUPPORTED_EXTENSIONS: phf::Map<&'static str, FileType> = phf_map! {
    "flac" => FileType::Flac,
    "mp3" => FileType::Mpeg,
};
