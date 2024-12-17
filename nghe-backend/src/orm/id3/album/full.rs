use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use diesel_async::RunQueryDsl;
use nghe_api::id3;
use uuid::Uuid;

use super::Album;
use crate::Error;
use crate::database::Database;
use crate::file::audio::duration::Trait as _;
use crate::orm::id3::{artist, song};
use crate::orm::songs;

#[derive(Debug, Queryable, Selectable)]
pub struct Full {
    #[diesel(embed)]
    pub album: Album,
    #[diesel(embed)]
    pub artists: artist::required::Artists,
    #[diesel(select_expression = sql("bool_or(songs_album_artists.compilation) is_compilation"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Bool>)]
    pub is_compilation: bool,
    #[diesel(select_expression = sql("array_agg(distinct(songs.id)) song_ids"))]
    #[diesel(select_expression_type = SqlLiteral::<sql_types::Array<sql_types::Uuid>>)]
    pub songs: Vec<Uuid>,
}

impl Full {
    pub async fn try_into(self, database: &Database) -> Result<id3::album::Full, Error> {
        let song = song::query::unchecked()
            .filter(songs::id.eq_any(self.songs))
            .get_results(&mut database.get().await?)
            .await?;
        let duration = song.duration();
        let song: Vec<_> = song.into_iter().map(song::Song::try_into).try_collect()?;

        let album = self
            .album
            .try_into_builder()?
            .song_count(song.len().try_into()?)
            .duration(duration.into())
            .build();

        Ok(id3::album::Full {
            album,
            artists: self.artists.into(),
            is_compilation: self.is_compilation,
            song,
        })
    }
}

pub mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;
    use crate::orm::id3::album;
    use crate::orm::{albums, permission, songs, songs_album_artists};

    #[auto_type]
    fn with_user_id_unchecked(user_id: Uuid) -> _ {
        let with_user_id_unchecked_no_group_by: album::query::with_user_id_unchecked_no_group_by =
            album::query::with_user_id_unchecked_no_group_by(user_id);
        let full: AsSelect<Full, crate::orm::Type> = Full::as_select();
        with_user_id_unchecked_no_group_by
            .inner_join(songs_album_artists::table.on(songs_album_artists::song_id.eq(songs::id)))
            .inner_join(artist::required::query::album())
            .group_by(albums::id)
            .select(full)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let with_user_id_unchecked: with_user_id_unchecked = with_user_id_unchecked(user_id);
        let permission: permission::with_album = permission::with_album(user_id);
        with_user_id_unchecked.filter(permission)
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
    use crate::file::audio;
    use crate::orm::albums;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_query(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(true, false)] allow: bool,
    ) {
        mock.add_music_folder().allow(allow).call().await;
        let mut music_folder = mock.music_folder(0).await;

        let album: audio::Album = Faker.fake();
        let album_id = album.upsert_mock(&mock, 0).await;
        music_folder.add_audio().album(album.clone()).n_song((2..4).fake()).call().await;

        let database_album = query::with_user_id(mock.user_id(0).await)
            .filter(albums::id.eq(album_id))
            .get_result(&mut mock.get().await)
            .await;

        if allow {
            let database_album = database_album.unwrap();
            assert_eq!(
                database_album.songs.iter().collect::<IndexSet<_>>(),
                music_folder.database.keys().collect::<IndexSet<_>>()
            );
        } else {
            assert!(database_album.is_err());
        }
    }
}
