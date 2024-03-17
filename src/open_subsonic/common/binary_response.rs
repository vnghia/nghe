use axum::{
    http::header,
    response::{IntoResponse, Response},
};

pub struct BinaryResponse {
    pub format: String,
    pub data: Vec<u8>,
}

impl BinaryResponse {
    fn get_content_type(&self) -> &'static str {
        match self.format.as_str() {
            "flac" => "audio/flac",
            "mp3" => "audio/mpeg",
            "opus" => "audio/opus",
            "ogg" | "oga" => "audio/ogg",
            "wav" => "audio/x-wav",
            "aac" | "m4a" => "audio/aac",
            _ => unreachable!("unsupported format encountered"),
        }
    }
}

impl IntoResponse for BinaryResponse {
    fn into_response(self) -> Response {
        let headers = [(header::CONTENT_TYPE, self.get_content_type())];
        (headers, self.data).into_response()
    }
}
