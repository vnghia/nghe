use std::net::{IpAddr, SocketAddr};

use educe::Educe;
use serde::{Deserialize, Serialize};
use typed_path::utils::utf8_current_dir;
use typed_path::Utf8PlatformPathBuf;

#[derive(Debug, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Server {
    #[educe(Default(expression = [127u8, 0u8, 0u8, 1u8].into()))]
    pub host: IpAddr,
    #[educe(Default(expression = 3000))]
    pub port: u16,
    #[serde(with = "crate::filesystem::path::serde")]
    #[educe(Default(expression =
        utf8_current_dir()
            .unwrap()
            .join("frontend")
            .join("dist")
            .with_platform_encoding_checked()
            .unwrap()
    ))]
    pub frontend_dir: Utf8PlatformPathBuf,
}

impl Server {
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
