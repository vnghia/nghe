#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub struct SongProperty {
    pub duration: u32,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub channel_count: u8,
}
