use diesel::prelude::*;
use nghe_api::id3;
use nghe_api::id3::builder::album as builder;

use super::Album;
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
    pub fn try_into_api_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        Ok(self
            .album
            .try_into_api_builder()?
            .song_count(self.durations.count().try_into()?)
            .duration(self.durations.sum()?))
    }

    pub fn try_into_api(self) -> Result<id3::album::Album, Error> {
        Ok(self.try_into_api_builder()?.build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::id3::album;

    #[auto_type]
    pub fn unchecked() -> _ {
        let with_durations: AsSelect<WithDurations, crate::orm::Type> = WithDurations::as_select();
        album::query::unchecked().select(with_durations)
    }
}

#[cfg(test)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use num_traits::ToPrimitive as _;
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
        let duration: f32 =
            music_folder.database.values().map(|information| information.property.duration).sum();

        let database_album = query::unchecked()
            .filter(albums::id.eq(album_id))
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        assert_eq!(database_album.durations.count(), n_song);
        assert_eq!(database_album.durations.sum().unwrap(), duration.ceil().to_u32().unwrap());
        assert_eq!(database_album.album.genres.value.len(), n_genre);
    }
}
