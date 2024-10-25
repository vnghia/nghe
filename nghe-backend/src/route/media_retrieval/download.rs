pub use nghe_api::media_retrieval::download::Request;
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::filesystem::{Filesystem, Trait};
use crate::response::{binary, Binary};
use crate::Error;

#[handler(role = download)]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    user_id: Uuid,
    request: Request,
) -> Result<Binary, Error> {
    let (filesystem, source) =
        binary::Source::audio(database, filesystem, user_id, request.id).await?;
    filesystem.read_to_binary(&source, None).await
}
