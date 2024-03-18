use crate::{models::*, OSError, ServerError};

use anyhow::Result;
use axum::http::{Request, Response};
use concat_string::concat_string;
use std::collections::HashMap;
use tower_http::services::{fs::ServeFileSystemResponseBody, ServeDir};
use urlencoding::encode;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServeMusicFolder(ServeDir);

pub type ServeMusicFolders = HashMap<Uuid, ServeMusicFolder>;
pub type ServeMusicFolderResult = Result<Response<ServeFileSystemResponseBody>>;
pub type ServeMusicFolderResponse = Result<Response<ServeFileSystemResponseBody>, ServerError>;

impl ServeMusicFolder {
    pub fn new(music_folders: Vec<music_folders::MusicFolder>) -> ServeMusicFolders {
        music_folders
            .into_iter()
            .map(|mf| {
                (
                    mf.id,
                    ServeMusicFolder(
                        ServeDir::new(mf.path).append_index_html_on_directories(false),
                    ),
                )
            })
            .collect()
    }

    pub async fn call(&mut self, path: &str) -> Result<Response<ServeFileSystemResponseBody>> {
        let req = Request::builder()
            .uri(concat_string!("http://localhost/", encode(path)))
            .body(())
            .expect("could not construct an empty request to call serve dir");
        self.0
            .try_call(req)
            .await
            .map_err(|err| OSError::from(err).into())
    }
}
