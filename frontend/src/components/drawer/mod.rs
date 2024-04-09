use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Drawer() -> Element {
    rsx! {
      div { class: "drawer lg:drawer-open",
        input {
          r#type: "checkbox",
          class: "drawer-toggle",
          id: "main-drawer-toggle"
        }
        div { class: "drawer-content flex flex-col",
          div { class: "w-full navbar bg-base-300",
            div { class: "navbar-start",
              div { class: "flex-none lg:hidden",
                label {
                  "aria-label": "open sidebar",
                  r#for: "main-drawer-toggle",
                  class: "btn btn-square btn-ghost",
                  svg {
                    "viewBox": "0 0 24 24",
                    "xmlns": "http://www.w3.org/2000/svg",
                    "fill": "none",
                    class: "inline-block w-6 h-6 stroke-current",
                    path {
                      "stroke-linecap": "round",
                      "stroke-width": "2",
                      "stroke-linejoin": "round",
                      "d": "M4 6h16M4 12h16M4 18h16"
                    }
                  }
                }
              }
            }
            div { class: "navbar-center",
              a { class: "text-base-content btn btn-ghost text-xl",
                "nghe"
              }
            }
            div { class: "navbar-end",
              button { class: "btn btn-ghost btn-circle",
                svg {
                  "xmlns": "http://www.w3.org/2000/svg",
                  "fill": "none",
                  "stroke": "currentColor",
                  "viewBox": "0 0 24 24",
                  class: "h-5 w-5",
                  path {
                    "stroke-linecap": "round",
                    "d": "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
                    "stroke-linejoin": "round",
                    "stroke-width": "2"
                  }
                }
              }
            }
          }
          Outlet::<Route> {}
        }
        div { class: "drawer-side",
          label {
            r#for: "main-drawer-toggle",
            "aria-label": "close sidebar",
            class: "drawer-overlay"
          }
          ul { class: "menu p-4 w-80 min-h-full bg-base-200 text-base-content",
            li { a { "Sidebar Item 1" } }
            li { a { "Sidebar Item 2" } }
          }
        }
      }
    }
}
