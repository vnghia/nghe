use nghe_proc_macro::api_derive;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[api_derive(request = false, response = false)]
#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, Clone, Copy)]
pub enum OpenSubsonicCode {
    AGenericError = 0,
    RequiredParameterIsMissing = 10,
    WrongUsernameOrPassword = 40,
    InvalidApiKey = 44,
    UserIsNotAuthorizedForTheGivenOperation = 50,
    TheRequestedDataWasNotFound = 70,
}

#[api_derive]
pub struct Error {
    pub code: OpenSubsonicCode,
    pub message: String,
}
