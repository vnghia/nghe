use nghe_proc_macros::{add_common_convert, add_request_types_test};

use crate::id::MediaTypedId;

#[add_common_convert]
#[derive(Debug)]
pub struct GetCoverArtParams {
    pub id: MediaTypedId,
}

add_request_types_test!(GetCoverArtParams);
