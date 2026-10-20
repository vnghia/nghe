use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}
