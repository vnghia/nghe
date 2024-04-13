use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};

use super::MusicFolderPath;

#[add_common_convert]
pub struct GetFolderStatsParams {}

#[add_types_derive]
#[derive(Debug)]
pub struct FolderStats {
    pub music_folder: MusicFolderPath,
    pub artist_count: usize,
    pub album_count: usize,
    pub song_count: usize,
    pub user_count: usize,
    pub total_size: usize,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct GetFolderStatsBody {
    pub folder_stats: Vec<FolderStats>,
}
