#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative, fake::Dummy))]
#[cfg_attr(test, derivative(PartialEq))]
pub struct Property {
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    pub duration: f32,
    pub bitrate: u32,
    pub bit_depth: Option<u8>,
    pub sample_rate: u32,
    pub channel_count: u8,
}

#[cfg(test)]
mod test {
    use lofty::file::FileType;

    use super::*;

    impl Property {
        pub fn default(file_type: FileType) -> Self {
            match file_type {
                FileType::Flac => Self {
                    duration: 0f32,
                    bitrate: 585,
                    bit_depth: Some(24),
                    sample_rate: 32000,
                    channel_count: 2,
                },
                _ => unreachable!(),
            }
        }
    }
}
