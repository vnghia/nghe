use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};

use super::MusicFolderPath;

#[add_common_convert]
pub struct GetMusicFolderStatsParams {}

#[add_types_derive]
#[derive(Debug)]
pub struct MusicFolderStat {
    pub music_folder: MusicFolderPath,
    pub artist_count: u32,
    pub album_count: u32,
    pub song_count: u32,
    pub user_count: u32,
    pub total_size: u64,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct GetMusicFolderStatsBody {
    pub folder_stats: Vec<MusicFolderStat>,
}

add_request_types_test!(GetMusicFolderStatsParams);
