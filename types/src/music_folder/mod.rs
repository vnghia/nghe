pub mod get_folder_stats;

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
