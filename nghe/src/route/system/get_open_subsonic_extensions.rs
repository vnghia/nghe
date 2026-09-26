use nghe_api::system::get_open_subsonic_extensions::Extension;
pub use nghe_api::system::get_open_subsonic_extensions::{Request, Response};
use nghe_proc_macro::handler;

static EXTENSIONS: &[Extension] = &[
    Extension { name: "transcodeOffset", versions: &[1] },
    Extension { name: "songLyrics", versions: &[1] },
    Extension { name: "formPost", versions: &[1] },
    Extension { name: "apiKeyAuthentication", versions: &[1] },
];

#[handler(need_auth = false)]
pub fn handler() -> Response {
    Response { open_subsonic_extensions: EXTENSIONS }
}
