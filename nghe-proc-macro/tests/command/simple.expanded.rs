use conf::{Conf, Subcommands};
pub enum Action {
    Create(nghe_api::user::create::Request),
    List,
    Get(nghe_api::user::get::Request),
    Delete(nghe_api::user::delete::Request),
}
#[automatically_derived]
impl ::core::fmt::Debug for Action {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Action::Create(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Create", &__self_0)
            }
            Action::List => ::core::fmt::Formatter::write_str(f, "List"),
            Action::Get(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Get", &__self_0)
            }
            Action::Delete(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Delete", &__self_0)
            }
        }
    }
}
