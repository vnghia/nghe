use nghe_proc_macros::{add_common_convert, add_subsonic_response};
use uuid::Uuid;

use crate::open_subsonic::common::id3::response::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetSongParams {
    pub id: Uuid,
}

#[add_subsonic_response]
pub struct GetSongBody {
    pub song: SongId3,
}
