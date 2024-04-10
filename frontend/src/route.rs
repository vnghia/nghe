use dioxus::prelude::*;

use crate::components::*;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Error)]
        #[layout(Drawer)]
            #[route("/")]
            Home {},
        #[end_layout]
        #[route("/setup")]
        Setup {},
        #[route("/login")]
        Login {},
}
