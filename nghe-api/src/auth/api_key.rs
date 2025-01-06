use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ApiKey {
    pub api_key: Uuid,
}

mod convert {
    use super::*;

    impl From<Uuid> for ApiKey {
        fn from(value: Uuid) -> Self {
            Self { api_key: value }
        }
    }
}
