use nghe_proc_macro::api_derive;

#[api_derive(response = true)]
pub struct WithCount {
    pub value: String,
    pub song_count: u32,
    pub album_count: u32,
}