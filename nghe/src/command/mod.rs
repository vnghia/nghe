use clap::{Parser, Subcommand};

use crate::server;

#[derive(Parser)]
struct Entrypoint {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Debug, Default, Subcommand)]
enum Action {
    #[default]
    Start,
}

pub async fn entrypoint() {
    match Entrypoint::try_parse() {
        Ok(entrypoint) => {
            let action = entrypoint.action.unwrap_or_default();
            match action {
                Action::Start => server::start().await,
            }
        }
        Err(error) => error.exit(),
    }
}
