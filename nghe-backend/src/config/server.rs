use std::net::{IpAddr, SocketAddr};

use educe::Educe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Server {
    #[educe(Default(expression = [127u8, 0u8, 0u8, 1u8].into()))]
    pub host: IpAddr,
    #[educe(Default(expression = 3000))]
    pub port: u16,
}

impl Server {
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
