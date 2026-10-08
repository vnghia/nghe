mod auth;
mod media_retrieval;
mod music_folder;
mod permission;
mod system;
mod user;

use conf::{Conf, Subcommands};
use url::Url;

#[derive(Debug, Subcommands)]
enum Endpoint {
    MediaRetrieval {
        #[conf(subcommands)]
        action: media_retrieval::Action,
    },
    MusicFolder {
        #[conf(subcommands)]
        action: music_folder::Action,
    },
    Permission {
        #[conf(subcommands)]
        action: permission::Action,
    },
    User {
        #[conf(subcommands)]
        action: user::Action,
    },
    System {
        #[conf(subcommands)]
        action: system::Action,
    },
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
