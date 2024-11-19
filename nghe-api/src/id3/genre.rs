use nghe_proc_macro::api_derive;

#[api_derive(response = true)]
pub struct Genre {
    pub name: String,
}
