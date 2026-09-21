#[derive(Debug, deluxe::ParseMetaItem, darling::FromMeta, bon::Builder)]
pub struct Attribute {
    #[deluxe(default = false)]
    #[darling(default = || false)]
    #[builder(default = false)]
    internal: bool,
    #[deluxe(default = false)]
    #[darling(default = || false)]
    #[builder(default = false)]
    json: bool,
}

impl Attribute {
    pub fn form(&self) -> bool {
        !self.internal
    }

    pub fn json(&self) -> bool {
        self.json || self.internal
    }
}
