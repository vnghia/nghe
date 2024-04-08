use dioxus::prelude::*;

use crate::components::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/setup")]
    Setup {},
    #[route("/login")]
    Login {},
}
