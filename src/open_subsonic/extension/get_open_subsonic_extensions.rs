use nghe_proc_macros::add_axum_response;

add_axum_response!(GetOpenSubsonicExtensionsBody);

pub async fn get_open_subsonic_extensions_handler() -> GetOpenSubsonicExtensionsJsonResponse {
    Ok(axum::Json(
        GetOpenSubsonicExtensionsBody {
            open_subsonic_extensions: vec![
                OSExtension { name: "transcodeOffset".into(), versions: vec![1] },
                OSExtension { name: "songLyrics".into(), versions: vec![1] },
                OSExtension { name: "formPost".into(), versions: vec![1] },
            ],
        }
        .into(),
    ))
}
