use nghe_proc_macros::add_response_derive;
use uuid::Uuid;

#[add_response_derive]
#[derive(Debug)]
pub struct MusicFolder {
    pub id: Uuid,
    pub path: String,
}
