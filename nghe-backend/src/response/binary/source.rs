use axum_extra::headers::ETag;
use concat_string::concat_string;
use diesel_async::RunQueryDsl;
use o2o::o2o;
use typed_path::Utf8TypedPathBuf;
use uuid::Uuid;

use crate::database::Database;
use crate::file::{self, audio};
use crate::filesystem::{self, Filesystem};
use crate::orm::binary;
use crate::Error;

pub trait ToETag: ToString {
    fn to_etag(&self) -> Result<ETag, Error> {
        concat_string!("\"", self.to_string(), "\"")
            .parse()
            .map_err(color_eyre::Report::from)
            .map_err(Error::from)
    }
}

impl ToETag for u64 {}

#[derive(o2o)]
#[from_owned(file::Property<F>)]
#[where_clause(F: file::Mime)]
pub struct Property<F> {
    #[from(~.into())]
    pub hash: Option<u64>,
    pub size: u32,
    pub format: F,
}

pub struct Source<F: file::Mime> {
    pub path: Utf8TypedPathBuf,
    pub property: Property<F>,
}

impl<F: file::Mime> Property<F> {
    pub fn mime(&self) -> &'static str {
        self.format.mime()
    }
}

impl<F: file::Mime> Property<F> {
    pub fn etag(&self) -> Result<Option<ETag>, Error> {
        self.hash.as_ref().map(u64::to_etag).transpose()
    }
}

impl Source<audio::Format> {
    pub async fn audio<'fs>(
        database: &Database,
        filesystem: &'fs Filesystem,
        user_id: Uuid,
        song_id: Uuid,
    ) -> Result<(filesystem::Impl<'fs>, Self), Error> {
        let audio = binary::source::audio::query(user_id, song_id)
            .get_result(&mut database.get().await?)
            .await?;
        let filesystem = filesystem.to_impl(audio.music_folder.ty.into())?;
        let path = filesystem
            .path()
            .from_string(audio.music_folder.path.into_owned())
            .join(audio.relative_path);
        let property: file::Property<_> = audio.property.into();
        let property = property.into();
        Ok((filesystem, Self { path, property }))
    }
}
