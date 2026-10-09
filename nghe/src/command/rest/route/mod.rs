mod media_retrieval;
mod music_folder;
mod permission;
mod system;
mod user;

use conf::{Conf, Subcommands};

#[derive(Debug, Subcommands)]
pub enum Route {
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
