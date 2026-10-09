mod auth;
mod route;

use conf::Conf;
use url::Url;

#[derive(Debug, Conf)]
pub struct Rest {
    #[arg(long)]
    server: Option<Url>,
    #[conf(flatten)]
    auth: auth::Auth,
    #[conf(subcommands)]
    route: route::Route,
}
