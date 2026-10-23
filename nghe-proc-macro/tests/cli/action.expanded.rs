use conf::{Conf, Subcommands};
use crate::cli::rest::Runner;
use crate::cli::Error;
pub enum Action {
    Create(nghe_api::route::user::create::Request),
    List,
    Get(nghe_api::route::user::get::Request),
    Delete(nghe_api::route::user::delete::Request),
}
#[automatically_derived]
impl ::core::fmt::Debug for Action {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Self::Create(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Create", &__self_0)
            }
            Self::List => ::core::fmt::Formatter::write_str(f, "List"),
            Self::Get(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Get", &__self_0)
            }
            Self::Delete(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Delete", &__self_0)
            }
        }
    }
}
impl Action {
    pub async fn run(&self, runner: &Runner) -> Result<(), Error> {
        match self {
            Self::Create(request) => runner.run_endpoint(Some(request)).await,
            Self::List => {
                runner.run_endpoint::<nghe_api::route::user::list::Request>(None).await
            }
            Self::Get(request) => runner.run_binary(Some(request)).await,
            Self::Delete(request) => runner.run_endpoint(Some(request)).await,
        }
    }
}
