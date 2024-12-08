use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Educe)]
#[educe(Default)]
pub struct Scan {
    // 100 KiB in bytes
    #[educe(Default(expression = 100 * 1024))]
    pub minimum_size: usize,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[educe(Default(expression = Some(10)))]
    pub channel_size: Option<usize>,
    #[educe(Default(expression = 10))]
    pub pool_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Tls {
    #[educe(Default(expression = false))]
    pub accept_invalid_certs: bool,
    #[educe(Default(expression = false))]
    pub accept_invalid_hostnames: bool,
}

#[derive(Debug, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct S3 {
    #[educe(Default(expression =
        std::env::var("AWS_ACCESS_KEY_ID").is_ok() && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
    ))]
    pub enable: bool,
    #[educe(Default(expression = std::env::var("AWS_USE_PATH_STYLE_ENDPOINT").is_ok()))]
    pub use_path_style_endpoint: bool,
    #[educe(Default(expression = 15))]
    pub presigned_duration: u64,
    #[educe(Default(expression = 0))]
    pub stalled_stream_grace_preriod: u64,
    #[educe(Default(expression = 5))]
    pub connect_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Filesystem {
    pub scan: Scan,
    pub tls: Tls,
    pub s3: S3,
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl S3 {
        pub fn test() -> Self {
            Self { enable: true, ..Self::default() }
        }
    }

    impl Filesystem {
        pub fn test() -> Self {
            Self { s3: S3::test(), ..Self::default() }
        }
    }
}
