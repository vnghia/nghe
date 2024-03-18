use axum::{
    http::header,
    response::{IntoResponse, Response},
};

pub struct BinaryResponse {
    pub format: String,
    pub data: Vec<u8>,
}

impl IntoResponse for BinaryResponse {
    fn into_response(self) -> Response {
        let headers = [(
            header::CONTENT_TYPE,
            mime_guess::from_ext(&self.format)
                .first_or_octet_stream()
                .essence_str()
                .to_owned(),
        )];
        (headers, self.data).into_response()
    }
}
