use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive(response = true)]
pub struct Required {
    pub id: Uuid,
    pub name: String,
}
