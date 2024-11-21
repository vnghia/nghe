use diesel::prelude::*;
use nghe_api::id3;
use nghe_api::id3::builder::album as builder;

use super::Album;
use crate::orm::id3::duration::Trait as _;
use crate::orm::id3::song;
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
pub struct WithDurations {
    #[diesel(embed)]
    pub album: Album,
    #[diesel(embed)]
    pub durations: song::durations::Durations,
}

pub type BuilderSet = builder::SetDuration<builder::SetSongCount<super::BuilderSet>>;

impl WithDurations {
    pub fn try_into_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        Ok(self
            .album
            .try_into_builder()?
            .song_count(self.durations.count().try_into()?)
            .duration(self.durations.duration()?))
    }
}

impl TryFrom<WithDurations> for id3::album::Album {
    type Error = Error;

    fn try_from(value: WithDurations) -> Result<Self, Self::Error> {
        Ok(value.try_into_builder()?.build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};
    use uuid::Uuid;

    use super::*;
    use crate::orm::id3::album;
    use crate::orm::{albums, permission};

    #[auto_type]
    pub fn unchecked() -> _ {
        let with_durations: AsSelect<WithDurations, crate::orm::Type> = WithDurations::as_select();
        album::query::unchecked().select(with_durations)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let permission: permission::with_album = permission::with_album(user_id);
        unchecked().filter(permission)
    }

    #[auto_type]
    pub fn with_music_folder<'ids>(user_id: Uuid, music_folder_ids: &'ids [Uuid]) -> _ {
        let with_user_id: with_user_id = with_user_id(user_id);
        with_user_id.filter(albums::music_folder_id.eq_any(music_folder_ids))
    }
}

#[cfg(test)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::orm::albums;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(#[future(awt)] mock: Mock, #[values(0, 2)] n_genre: usize) {
        let mut music_folder = mock.music_folder(0).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;

        let n_song = (2..4).fake();
        music_folder
            .add_audio()
            .album(album)
            .genres(fake::vec![String; n_genre].into_iter().collect())
            .n_song(n_song)
            .call()
            .await;

        let database_album = query::unchecked()
            .filter(albums::id.eq(album_id))
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        assert_eq!(database_album.durations.count(), n_song);
        assert_eq!(
            database_album.durations.duration().unwrap(),
            music_folder.database.duration().unwrap()
        );
        assert_eq!(database_album.album.genres.value.len(), n_genre);
    }
}
