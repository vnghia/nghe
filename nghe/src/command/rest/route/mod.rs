mod media_retrieval;
mod music_folder;
mod permission;
mod system;
mod user;

use conf::{Conf, Subcommands};

use super::{Error, Runner};

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

impl Route {
    pub async fn run(&self, runner: &Runner) -> Result<(), Error> {
        match self {
            Route::MediaRetrieval { action } => action.run(runner).await,
            Route::MusicFolder { action } => action.run(runner).await,
            Route::Permission { action } => action.run(runner).await,
            Route::User { action } => action.run(runner).await,
            Route::System { action } => action.run(runner).await,
        }
    }
}
