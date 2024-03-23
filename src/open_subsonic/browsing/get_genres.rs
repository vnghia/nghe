use axum::extract::State;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;

use crate::Database;

#[add_validate]
#[derive(Debug)]
pub struct GetGenresParams {}

#[derive(Serialize)]
pub struct GenresResult {}

#[wrap_subsonic_response]
pub struct GenresBody {
    genres: GenresResult,
}

pub async fn get_genres_handler(
    State(_): State<Database>,
    _: GetGenresRequest,
) -> GenresJsonResponse {
    GenresBody { genres: GenresResult {} }.into()
}
