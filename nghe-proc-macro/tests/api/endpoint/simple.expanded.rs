use nghe_proc_macro::api_derive;
#[serde_with::apply(
    Option = >#[serde(skip_serializing_if = "Option::is_none", default)],
    Vec = >#[serde(skip_serializing_if = "Vec::is_empty", default)],
    date::Date = >#[serde(skip_serializing_if = "date::Date::is_none", default)],
    genre::Genres = >#[serde(skip_serializing_if = "genre::Genres::is_empty", default)],
    OffsetDateTime = >#[serde(with = "crate::time::serde")],
    Option<OffsetDateTime>= >#[serde(with = "crate::time::serde::option")],
    time::SignedDuration = >#[serde(with = "crate::time::signed_duration::serde")],
)]
#[serde(rename_all = "camelCase")]
#[endpoint(path = "path/endpoint")]
pub struct Request {
    pub token: bool,
}
#[automatically_derived]
impl ::core::fmt::Debug for Request {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Request",
            "token",
            &&self.token,
        )
    }
}
#[serde_with::apply(
    Option = >#[serde(skip_serializing_if = "Option::is_none", default)],
    Vec = >#[serde(skip_serializing_if = "Vec::is_empty", default)],
    date::Date = >#[serde(skip_serializing_if = "date::Date::is_none", default)],
    genre::Genres = >#[serde(skip_serializing_if = "genre::Genres::is_empty", default)],
    OffsetDateTime = >#[serde(with = "crate::time::serde")],
    Option<OffsetDateTime>= >#[serde(with = "crate::time::serde::option")],
    time::SignedDuration = >#[serde(with = "crate::time::signed_duration::serde")],
)]
#[serde(rename_all = "camelCase")]
pub struct AuthFormRequest<'auth_u, 'auth_c, 'auth_s, 'auth_p> {
    pub token: bool,
    #[serde(flatten, borrow)]
    auth: crate::auth::Form<'auth_u, 'auth_c, 'auth_s, 'auth_p>,
}
#[automatically_derived]
impl<'auth_u, 'auth_c, 'auth_s, 'auth_p> ::core::fmt::Debug
for AuthFormRequest<'auth_u, 'auth_c, 'auth_s, 'auth_p> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "AuthFormRequest",
            "token",
            &self.token,
            "auth",
            &&self.auth,
        )
    }
}
impl crate::common::FormURL for Request {
    const URL_FORM: &'static str = "/path/endpoint";
    const URL_FORM_VIEW: &'static str = "/path/endpoint.view";
}
impl<
    'u,
    'c,
    's,
    'p,
    'de: 'u + 'c + 's + 'p,
> crate::auth::form::Trait<'u, 'c, 's, 'p, 'de, Request>
for AuthFormRequest<'u, 'c, 's, 'p> {
    fn auth<'form>(&'form self) -> &'form crate::auth::Form<'u, 'c, 's, 'p> {
        &self.auth
    }
    fn new(request: Request, auth: crate::auth::Form<'u, 'c, 's, 'p>) -> Self {
        let Request { token } = request;
        Self { token, auth }
    }
    fn request(self) -> Request {
        let Self { token, auth } = self;
        Request { token }
    }
}
impl<
    'u,
    'c,
    's,
    'p,
    'de: 'u + 'c + 's + 'p,
> crate::common::FormRequest<'u, 'c, 's, 'p, 'de> for Request {
    type AuthForm = AuthFormRequest<'u, 'c, 's, 'p>;
}
impl crate::common::FormEndpoint for Request {
    type Response = Response;
}
