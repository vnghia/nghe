use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Scan {
    // 100 KiB in bytes
    #[derivative(Default(value = "102400"))]
    pub minimum_size: u64,
    #[derivative(Default(value = "10"))]
    pub channel_size: usize,
    #[derivative(Default(value = "10"))]
    pub pool_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Tls {
    #[derivative(Default(value = "false"))]
    pub accept_invalid_certs: bool,
    #[derivative(Default(value = "false"))]
    pub accept_invalid_hostnames: bool,
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct S3 {
    #[derivative(Default(value = "std::env::var(\"AWS_ACCESS_KEY_ID\").is_ok() && \
                                  std::env::var(\"AWS_SECRET_ACCESS_KEY\").is_ok()"))]
    pub enable: bool,
    #[derivative(Default(value = "std::env::var(\"AWS_USE_PATH_STYLE_ENDPOINT\").is_ok()"))]
    pub use_path_style_endpoint: bool,
    #[derivative(Default(value = "15"))]
    pub presigned_url_duration: u64,
    #[derivative(Default(value = "0"))]
    pub stalled_stream_grace_preriod: u64,
    #[derivative(Default(value = "5"))]
    pub connect_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Filesystem {
    pub scan: Scan,
    pub tls: Tls,
    pub s3: S3,
}
