mod auth;
mod client;
mod route;

use conf::Conf;
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
    #[conf(subcommands)]
    route: route::Route,
}

impl Rest {
    pub async fn run(&self) -> Result<(), Error> {
        let client = client::Client::new(&self.server)?;
        Ok(())
    }
}
