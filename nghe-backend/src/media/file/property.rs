#[derive(Debug)]
pub struct Property {
    pub duration: f32,
    pub bitrate: u32,
    pub bit_depth: Option<u8>,
    pub sample_rate: u32,
    pub channel_count: u8,
}
