pub mod full;
pub mod permission;
pub mod short;

use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types;
use nghe_api::playlists::playlist::{self, builder};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::orm::playlists;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = playlists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = false)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub comment: Option<String>,
    pub public: bool,
    #[diesel(column_name = created_at)]
    pub created: OffsetDateTime,
    #[diesel(select_expression = sql(
        "greatest(max(playlists_songs.created_at), playlists.updated_at)"
    ))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Timestamptz>)]
    pub changed: OffsetDateTime,
}

pub type BuilderSet = builder::SetChanged<
    builder::SetCreated<builder::SetPublic<builder::SetComment<builder::SetName<builder::SetId>>>>,
>;

impl Playlist {
    pub fn into_builder(self) -> builder::Builder<BuilderSet> {
        playlist::Playlist::builder()
            .id(self.id)
            .name(self.name)
            .comment(self.comment)
            .public(self.public)
            .created(self.created)
            .changed(self.changed)
    }
}

pub mod query {
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;
    use crate::orm::{albums, permission, playlists, playlists_songs, playlists_users, songs};

    #[auto_type]
    pub fn with_album(user_id: Uuid) -> _ {
        let permission: permission::with_album = permission::with_album(user_id);
        albums::id.is_null().or(permission)
    }

    #[auto_type]
    pub fn with_user_id(user_id: Uuid) -> _ {
        let with_album: with_album = with_album(user_id);
        let playlist: AsSelect<Playlist, crate::orm::Type> = Playlist::as_select();
        playlists::table
            .inner_join(
                playlists_users::table.on(playlists_users::playlist_id
                    .eq(playlists::id)
                    .and(playlists_users::user_id.eq(user_id))),
            )
            .left_join(playlists_songs::table)
            .left_join(songs::table.on(songs::id.eq(playlists_songs::song_id)))
            .left_join(albums::table.on(albums::id.eq(songs::album_id)))
            .filter(with_album)
            .group_by(playlists::id)
            .select(playlist)
    }
}

#[cfg(test)]
#[coverage(off)]
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
        if n_song == 0 {
            assert_eq!(database_playlist.created, database_playlist.changed);
        } else {
            assert!(database_playlist.changed > database_playlist.created);
        }
    }
}
