use std::borrow::Cow;

use o2o::o2o;
use uuid::Uuid;

use super::Genres;
use crate::database::Database;
use crate::orm::songs;
use crate::orm::upsert::Upsert as _;
use crate::{file, Error};

#[derive(Debug, o2o)]
#[ref_try_into(songs::Data<'a>, Error)]
#[cfg_attr(test, derive(fake::Dummy, Clone))]
pub struct Information<'a> {
    #[ref_into(songs::Data<'a>| song, (&~.song).try_into()?)]
    pub metadata: super::Metadata<'a>,
    #[map(~.try_into()?)]
    pub property: super::Property,
    #[map(~.into())]
    pub file: file::Property<super::Format>,
}

impl<'a> Information<'a> {
    pub async fn upsert_album(
        &self,
        database: &Database,
        music_folder_id: Uuid,
    ) -> Result<Uuid, Error> {
        self.metadata.album.upsert(database, music_folder_id).await
    }

    pub async fn upsert_genres(&self, database: &Database, song_id: Uuid) -> Result<(), Error> {
        let genre_ids = self.metadata.genres.upsert(database).await?;
        Genres::upsert_song(database, song_id, &genre_ids).await
    }

    pub async fn upsert(
        &self,
        database: &Database,
        album_id: Uuid,
        relative_path: impl Into<Cow<'_, str>>,
        id: impl Into<Option<Uuid>>,
    ) -> Result<Uuid, Error> {
        songs::Upsert { album_id, relative_path: relative_path.into(), data: self.try_into()? }
            .upsert(database, id)
            .await
    }
}
