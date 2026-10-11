use conf::{Conf, Subcommands};
use crate::command::rest::Runner;
use crate::command::Error;
pub enum Route {
    MusicFolder { #[conf(subcommands)] action: music_folder::Action },
    User { #[conf(subcommands)] action: user::Action },
}
#[automatically_derived]
impl ::core::fmt::Debug for Route {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Route::MusicFolder { action: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "MusicFolder",
                    "action",
                    &__self_0,
                )
            }
            Route::User { action: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "User",
                    "action",
                    &__self_0,
                )
            }
        }
    }
}
impl Route {
    pub async fn run(&self, runner: &Runner) -> Result<(), Error> {
        match self {
            Self::MusicFolder { action } => action.run(runner).await,
            Self::User { action } => action.run(runner).await,
        }
    }
}
