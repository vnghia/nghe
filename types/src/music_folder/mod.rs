pub mod add_music_folder;
pub mod get_music_folder_stats;
pub mod remove_music_folder;
pub mod update_music_folder;

use nghe_proc_macros::add_types_derive;
use uuid::Uuid;

#[add_types_derive]
#[derive(Debug)]
pub struct MusicFolder {
    pub id: Uuid,
    pub name: String,
}

#[add_types_derive]
#[derive(Debug)]
pub struct MusicFolderPath {
    pub id: Uuid,
    pub name: String,
    pub path: String,
}
