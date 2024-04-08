use nghe_proc_macros::{add_response_derive, add_subsonic_response};

#[add_response_derive]
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
