mod auth;
mod route;
mod runner;

use conf::Conf;
use route::Route;
use runner::Runner;
use typed_path::Utf8PlatformPathBuf;
use url::Url;

use super::Error;

#[derive(Debug, Conf)]
pub struct Server {
    #[conf(long)]
    pub url: Option<Url>,
    #[conf(flatten)]
    pub auth: auth::Auth,
}

#[derive(Debug, Conf)]
pub struct Rest {
    #[conf(flatten)]
    server: Server,
    #[conf(long)]
    output: Option<Utf8PlatformPathBuf>,
    #[conf(subcommands)]
    route: Route,
}

impl Rest {
    pub async fn run(self) -> Result<(), Error> {
        Runner::try_from(self)?.run().await
    }
}
