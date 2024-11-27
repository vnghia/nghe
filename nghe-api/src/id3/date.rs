use nghe_proc_macro::api_derive;

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
)]
#[api_derive]
#[derive(Default)]
pub struct Date {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl Date {
    pub fn is_none(&self) -> bool {
        self.year.is_none()
    }
}
