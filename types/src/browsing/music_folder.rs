use nghe_proc_macros::add_types_derive;
use uuid::Uuid;

#[add_types_derive]
#[derive(Debug)]
pub struct MusicFolder {
    pub id: Uuid,
    pub path: String,
}
