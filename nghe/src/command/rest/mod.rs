mod auth;

use clap::Args;
use url::Url;

#[derive(Debug, Args)]
pub struct Rest {
    #[arg(long)]
    server: Option<Url>,
    #[command(flatten)]
    auth: auth::Auth,
}
