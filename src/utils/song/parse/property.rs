use lofty::file::FileType;

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(Default))]
pub struct SongProperty {
    #[cfg_attr(test, derivative(Default(value = "FileType::Flac")))]
    pub format: FileType,
    pub duration: f32,
    pub bitrate: u32,
    pub bit_depth: Option<u8>,
    pub sample_rate: u32,
    pub channel_count: u8,
}
