#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(educe::Educe, fake::Dummy))]
#[cfg_attr(test, educe(PartialEq, Eq))]
pub struct Property {
    #[cfg_attr(test, educe(PartialEq(ignore)))]
    #[cfg_attr(test, dummy(faker = "100f32..300f32"))]
    pub duration: f32,
    #[cfg_attr(test, dummy(faker = "32000..640000"))]
    pub bitrate: u32,
    pub bit_depth: Option<u8>,
    #[cfg_attr(test, dummy(faker = "10000..44000"))]
    pub sample_rate: u32,
    pub channel_count: u8,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file::audio;

    impl Property {
        pub fn default(ty: audio::Format) -> Self {
            match ty {
                audio::Format::Flac => Self {
                    duration: 0f32,
                    bitrate: 585,
                    bit_depth: Some(24),
                    sample_rate: 32000,
                    channel_count: 2,
                },
            }
        }
    }
}
