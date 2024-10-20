use std::net::{IpAddr, SocketAddr};

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use typed_path::utils::utf8_current_dir;
use typed_path::Utf8TypedPathBuf;

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct Server {
    #[derivative(Default(value = "[127u8, 0u8, 0u8, 1u8].into()"))]
    pub host: IpAddr,
    #[derivative(Default(value = "3000"))]
    pub port: u16,
    #[serde(with = "crate::filesystem::path::serde")]
    #[derivative(Default(
        value = "utf8_current_dir().unwrap().join(\"frontend\").join(\"dist\").to_typed_path_buf()"
    ))]
    pub frontend_dir: Utf8TypedPathBuf,
}

impl Server {
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
