use axum::extract::State;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use nghe_types::id3::InfoId3;

use crate::Database;

add_common_validate!(GetAlbumInfo2Params);
add_axum_response!(GetAlbumInfo2Body);

pub async fn get_album_info2_handler(
    State(_): State<Database>,
    _: GetAlbumInfo2Request,
) -> GetAlbumInfo2JsonResponse {
    Ok(axum::Json(
        GetAlbumInfo2Body {
            album_info: AlbumInfo {
                notes: None,
                info: InfoId3 {
                    music_brainz_id: None,
                    last_fm_url: None,
                    small_image_url: None,
                    medium_image_url: None,
                    large_image_url: None,
                },
            },
        }
        .into(),
    ))
}
