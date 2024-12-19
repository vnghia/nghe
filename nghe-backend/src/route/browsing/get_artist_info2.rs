use std::borrow::Cow;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::browsing::get_artist_info2::ArtistInfo2;
pub use nghe_api::browsing::get_artist_info2::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::{artist_informations, artists, id3};

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let music_brainz_id: Option<Uuid> = id3::artist::query::with_user_id(user_id)
        .filter(artists::id.eq(request.id))
        .select(artists::mbz_id)
        .get_result(&mut database.get().await?)
        .await?;
    let lastfm = artist_informations::table
        .filter(artist_informations::artist_id.eq(request.id))
        .select(artist_informations::Lastfm::as_select())
        .get_result(&mut database.get().await?)
        .await
        .optional()?;

    Ok(Response {
        artist_info2: if let Some(lastfm) = lastfm {
            ArtistInfo2 {
                music_brainz_id: music_brainz_id.or(lastfm.mbz_id),
                lastfm_url: lastfm.url.map(Cow::into_owned),
                biography: lastfm.biography.map(Cow::into_owned),
            }
        } else {
            ArtistInfo2 { music_brainz_id, ..Default::default() }
        },
    })
}
