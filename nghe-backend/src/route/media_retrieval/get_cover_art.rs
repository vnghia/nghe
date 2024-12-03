use axum_extra::headers::Range;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
pub use nghe_api::media_retrieval::get_cover_art::Request;
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::file::{self, picture};
use crate::http::binary;
use crate::http::header::ToOffset;
use crate::orm::cover_arts;
use crate::{config, Error};

#[handler]
pub async fn handler(
    database: &Database,
    #[handler(header)] range: Option<Range>,
    config: config::CoverArt,
    request: Request,
) -> Result<binary::Response, Error> {
    let dir = &config.dir.ok_or_else(|| Error::MediaCoverArtDirIsNotEnabled)?;
    let property: file::Property<picture::Format> = cover_arts::table
        .filter(cover_arts::id.eq(request.id))
        .select(cover_arts::Property::as_select())
        .get_result(&mut database.get().await?)
        .await?
        .into();
    let offset = range.map(|range| range.to_offset(property.size.into())).transpose()?;
    binary::Response::from_path_property(
        property.path(dir, picture::Picture::FILENAME),
        &property,
        offset,
        #[cfg(test)]
        None,
    )
    .await
}
