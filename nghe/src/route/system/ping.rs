pub use nghe_api::route::system::ping::{Request, Response};
use nghe_proc_macro::handler;

#[handler]
pub fn handler() -> Response {
    Response
}
