use nghe_proc_macros::{add_subsonic_response, add_types_derive};

#[add_types_derive]
#[derive(Debug)]
pub struct ActualError {
    pub code: u8,
    pub message: String,
}

#[add_subsonic_response(success = false)]
#[derive(Debug)]
pub struct ErrorBody {
    pub error: ActualError,
}
