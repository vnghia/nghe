use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_async::RunQueryDsl;
use nghe_api::playlists::playlist;
use uuid::Uuid;

use super::Playlist;
use crate::Error;
use crate::database::Database;
use crate::file::audio::duration::Trait as _;
use crate::orm::id3::song;
use crate::orm::{playlists_songs, songs};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists, check_for_backend(crate::orm::Type))]
pub struct Full {
    #[diesel(embed)]
    pub playlist: Playlist,
    #[diesel(select_expression = sql(
        "array_remove(array_agg(distinct(songs.id)), null) entry_ids"
    ))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub entries: Vec<Uuid>,
}

impl Full {
    pub async fn try_into(
        self,
        database: &Database,
        user_id: Uuid,
    ) -> Result<playlist::Full, Error> {
        let entry = song::short::query::with_user_id_unchecked(user_id)
            .inner_join(playlists_songs::table)
            .filter(songs::id.eq_any(self.entries))
            .filter(playlists_songs::playlist_id.eq(self.playlist.id))
            .order_by(sql::<sql_types::Timestamptz>("any_value(playlists_songs.created_at)"))
            .get_results(&mut database.get().await?)
            .await?;
        let duration = entry.duration();
        let entry: Vec<_> = entry.into_iter().map(song::short::Short::try_into).try_collect()?;

        let playlist = self
            .playlist
            .into_builder()
            .song_count(entry.len().try_into()?)
            .duration(duration.into())
            .build();

        Ok(playlist::Full { playlist, entry })
    }
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::playlist;

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let with_user_id: playlist::query::with_user_id = playlist::query::with_user_id(user_id);
        let full: AsSelect<Full, crate::orm::Type> = Full::as_select();
        with_user_id.select(full)
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use indexmap::IndexSet;
    use rstest::rstest;

    use super::*;
    use crate::route::playlists::create_playlist;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(#[future(awt)] mock: Mock, #[values(0, 5)] n_song: usize) {
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio().n_song(n_song).call().await;

        let user_id = mock.user_id(0).await;
        create_playlist::handler(
            mock.database(),
            user_id,
            create_playlist::Request {
                create_or_update: Faker.fake::<String>().into(),
                song_ids: Some(music_folder.database.keys().copied().collect()),
            },
        )
        .await
        .unwrap();

        let database_playlist =
            query::with_user_id(user_id).get_result(&mut mock.get().await).await.unwrap();
        assert_eq!(
            database_playlist.entries.iter().collect::<IndexSet<_>>(),
            music_folder.database.keys().collect::<IndexSet<_>>()
        );
    }
}
