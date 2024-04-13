use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};

use super::MusicFolderPath;

#[add_common_convert]
pub struct GetFolderStatsParams {}

#[add_types_derive]
#[derive(Debug)]
pub struct FolderStats {
    pub music_folder: MusicFolderPath,
    pub artist_count: u32,
    pub album_count: u32,
    pub song_count: u32,
    pub user_count: u32,
    pub total_size: u64,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct GetFolderStatsBody {
    pub folder_stats: Vec<FolderStats>,
}
