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
#[conf(env_prefix = "NGHE_CLI_REST_")]
pub struct Rest {
    #[conf(long, env)]
    pub url: Option<Url>,
    /// Either <username:password> for basic auth or arbitrary string for bearer auth
    #[conf(long, env)]
    pub auth: Option<auth::Auth>,
    #[conf(long, env)]
    output: Option<Utf8PlatformPathBuf>,
    #[conf(subcommands)]
    route: Route,
}

impl Rest {
    pub async fn run(self) -> Result<(), Error> {
        Runner::try_from(self)?.run().await
    }
}
