use dioxus::prelude::*;
use uuid::Uuid;

use crate::components::*;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Global)]
        #[layout(Drawer)]
            #[route("/")]
            Home {},
            #[route("/users")]
            Users {},
            #[route("/user/create")]
            CreateUser {},
            #[route("/folders")]
            Folders {},
            #[route("/folder/add")]
            AddFolder {},
            #[route("/folder/:id")]
            Folder {id: Uuid},
        #[end_layout]
        #[route("/setup")]
        Setup {},
        #[route("/login")]
        Login {},
}
