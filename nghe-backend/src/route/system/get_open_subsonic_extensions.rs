use nghe_api::system::get_open_subsonic_extensions::Extension;
pub use nghe_api::system::get_open_subsonic_extensions::{Request, Response};
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::Error;

static EXTENSIONS: &[Extension] = &[
    Extension { name: "transcodeOffset", versions: &[1] },
    Extension { name: "songLyrics", versions: &[1] },
    Extension { name: "formPost", versions: &[1] },
];

#[handler(need_auth = false)]
pub async fn handler(_database: &Database, request: Request) -> Result<Response, Error> {
    Ok(Response { open_subsonic_extensions: EXTENSIONS })
}
