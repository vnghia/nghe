use std::net::{IpAddr, SocketAddr};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::fs::path::LocalPathBuf;

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct Server {
    #[derivative(Default(value = "[127u8, 0u8, 0u8, 1u8].into()"))]
    pub host: IpAddr,
    #[derivative(Default(value = "3000"))]
    pub port: u16,
    #[serde(with = "crate::fs::path::serde")]
    #[derivative(Default(value = "std::env::current_dir().unwrap().join(\"frontend\").join(\"\
                                  dist\").into_os_string().into_string().expect(\"Non UTF-8 \
                                  path encountered\").into()"))]
    pub frontend_dir: LocalPathBuf,
}

impl Server {
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
