pub use nghe_api::system::health::{Request, Response};
use nghe_proc_macro::handler;

#[handler(need_auth = false)]
pub fn handler() -> Response {
    Response
}
