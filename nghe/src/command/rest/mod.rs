mod auth;

use conf::{Conf, Subcommands};
use url::Url;

#[derive(Debug, Subcommands)]
enum Endpoint {
    // user
    UserSetup(nghe_api::user::setup::Request),
}

#[derive(Debug, Conf)]
pub struct Rest {
    #[arg(long)]
    server: Option<Url>,
    #[conf(flatten)]
    auth: auth::Auth,
    #[conf(subcommands)]
    endpoint: Endpoint,
}
