use nghe_proc_macros::{add_axum_response, add_subsonic_response};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OSExtension {
    name: &'static str,
    versions: &'static [u8],
}

#[add_subsonic_response]
pub struct GetOpenSubsonicExtensionsBody {
    open_subsonic_extensions: &'static [OSExtension],
}
add_axum_response!(GetOpenSubsonicExtensionsBody);

pub async fn get_open_subsonic_extensions_handler() -> GetOpenSubsonicExtensionsJsonResponse {
    GetOpenSubsonicExtensionsBody {
        open_subsonic_extensions: &[
            OSExtension { name: "transcodeOffset", versions: &[1] },
            OSExtension { name: "songLyrics", versions: &[1] },
        ],
    }
    .into()
}
