pub use nghe_api::system::ping::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;

#[handler]
pub async fn handler(request: Request) -> Result<Response, Error> {
    Ok(Response)
}
