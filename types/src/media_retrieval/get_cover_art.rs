use nghe_proc_macros::add_common_convert;

use crate::id::MediaTypedId;

#[add_common_convert]
#[derive(Debug)]
pub struct GetCoverArtParams {
    pub id: MediaTypedId,
}
