#![allow(clippy::needless_pass_by_value)]

mod generic;
mod http;

use leptos::either::Either;
use leptos::prelude::*;

fn Error(errors: ArcRwSignal<Errors>) -> impl IntoView {
    let errors = errors();
    leptos::logging::error!("{:?}", errors);
    errors.into_iter().next().map(|(_, error)| {
        let error = error.into_inner();
        match error.downcast_ref::<crate::Error>().expect("Could not handle this error type") {
            crate::Error::Http(error) => Either::Left(http::Http(error.clone())),
            crate::Error::GlooNet(_) => Either::Right(generic::Generic(error.to_string())),
        }
    })
}

pub fn Boundary(child: TypedChildren<impl IntoView + 'static>) -> impl IntoView {
    ErrorBoundary(component_props_builder(&ErrorBoundary).fallback(Error).children(child).build())
}
