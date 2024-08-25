use nghe_proc_macro::api_derive;

#[api_derive(response = false)]
pub struct Role {
    pub admin: bool,
    pub stream: bool,
    pub download: bool,
    pub share: bool,
}
