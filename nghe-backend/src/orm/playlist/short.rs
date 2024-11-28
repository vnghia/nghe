use diesel::prelude::*;
use nghe_api::playlists::playlist::{self, builder};

use super::Playlist;
use crate::orm::id3::duration::Trait as _;
use crate::orm::id3::song;
use crate::Error;

#[derive(Debug, Queryable, Selectable)]
pub struct Short {
    #[diesel(embed)]
    pub playlist: Playlist,
    #[diesel(embed)]
    pub durations: song::durations::Durations,
}

pub type BuilderSet = builder::SetDuration<builder::SetSongCount<super::BuilderSet>>;

impl Short {
    pub fn try_into_builder(self) -> Result<builder::Builder<BuilderSet>, Error> {
        Ok(self
            .playlist
            .into_builder()
            .song_count(self.durations.count().try_into()?)
            .duration(self.durations.duration()?))
    }
}

impl TryFrom<Short> for playlist::Playlist {
    type Error = Error;

    fn try_from(value: Short) -> Result<Self, Self::Error> {
        Ok(value.try_into_builder()?.build())
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::playlist;

    #[auto_type]
    pub fn unchecked() -> _ {
        let short: AsSelect<Short, crate::orm::Type> = Short::as_select();
        playlist::query::unchecked().select(short)
    }
}

#[cfg(test)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::route::playlists::create_playlist;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(#[future(awt)] mock: Mock, #[values(0, 5)] n_song: usize) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio().n_song(n_song).call().await;

        create_playlist::handler(
            mock.database(),
            mock.user_id(0).await,
            create_playlist::Request {
                create_or_update: Faker.fake::<String>().into(),
                song_ids: Some(music_folder.database.keys().copied().collect()),
            },
        )
        .await
        .unwrap();

        let database_playlist = query::unchecked().get_result(&mut mock.get().await).await.unwrap();
        assert_eq!(database_playlist.durations.count(), n_song);
        assert_eq!(
            database_playlist.durations.duration().unwrap(),
            music_folder.database.duration().unwrap()
        );
    }
}
