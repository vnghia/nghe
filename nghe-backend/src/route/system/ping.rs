pub use nghe_api::system::ping::{Request, Response};
use nghe_proc_macro::handler;

// TODO: add user auth
#[handler(need_auth = false)]
pub fn handler() -> Response {
    Response
}
