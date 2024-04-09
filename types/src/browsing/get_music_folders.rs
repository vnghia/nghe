use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};

use super::MusicFolder;

#[add_common_convert]
pub struct GetMusicFoldersParams {}

#[add_types_derive]
#[derive(Debug)]
pub struct MusicFolders {
    pub music_folder: Vec<MusicFolder>,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct GetMusicFoldersBody {
    pub music_folders: MusicFolders,
}
