mod error;
mod rest;

use conf::{Conf, Subcommands};
use error::Error;
use rest::Rest;

use crate::server;

#[derive(Debug, Default, Subcommands)]
enum Action {
    #[default]
    Start,
    Rest(Rest),
}

#[derive(Conf)]
struct Entrypoint {
    #[conf(subcommands)]
    action: Option<Action>,
}

pub async fn entrypoint() {
    let entrypoint = Entrypoint::parse();
    let action = entrypoint.action.unwrap_or_default();
    match action {
        Action::Start => server::start().await,
        Action::Rest(rest) => {
            rest.run().await.unwrap();
        }
    }
}
