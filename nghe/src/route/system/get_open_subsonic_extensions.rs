pub use nghe_api::route::system::get_open_subsonic_extensions::{Request, Response};
use nghe_proc_macro::handler;

#[handler]
pub fn handler() -> Response {
    Response { open_subsonic_extensions: () }
}
