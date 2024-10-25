use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::download::Request;
use nghe_proc_macro::handler;
use uuid::Uuid;

use super::offset;
use crate::database::Database;
use crate::filesystem::{Filesystem, Trait};
use crate::response::{binary, Binary};
use crate::Error;

#[handler(role = download, headers = [range])]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    range: Option<Range>,
    user_id: Uuid,
    request: Request,
) -> Result<Binary, Error> {
    let (filesystem, source) =
        binary::Source::audio(database, filesystem, user_id, request.id).await?;
    let offset = offset::from_range(range, source.property.size.into())?;
    filesystem.read_to_binary(&source, offset).await
}
