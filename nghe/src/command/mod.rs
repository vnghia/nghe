mod rest;

use clap::{Parser, Subcommand};
use rest::Rest;

use crate::server;

#[derive(Debug, Default, Subcommand)]
enum Action {
    #[default]
    Start,
    Rest(Rest),
}

#[derive(Parser)]
struct Entrypoint {
    #[command(subcommand)]
    action: Option<Action>,
}

pub async fn entrypoint() {
    match Entrypoint::try_parse() {
        Ok(entrypoint) => {
            let action = entrypoint.action.unwrap_or_default();
            match action {
                Action::Start => server::start().await,
                Action::Rest(rest) => {
                    dbg!(rest);
                }
            }
        }
        Err(error) => error.exit(),
    }
}
