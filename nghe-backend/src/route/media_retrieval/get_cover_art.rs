use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::get_cover_art::Request;
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::file::{self, picture};
use crate::http::binary;
use crate::http::header::ToOffset;
use crate::{Error, config, error};

#[handler]
pub async fn handler(
    database: &Database,
    #[handler(header)] range: Option<Range>,
    config: config::CoverArt,
    request: Request,
) -> Result<binary::Response, Error> {
    let dir = &config.dir.ok_or_else(|| error::Kind::MissingCoverArtDirectoryConfig)?;
    let property = file::Property::query_cover_art(database, request.id).await?;
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
