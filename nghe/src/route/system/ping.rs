pub use nghe_api::system::ping::{Request, Response};
use nghe_proc_macro::handler;

#[handler]
pub fn handler() -> Response {
    Response
}
