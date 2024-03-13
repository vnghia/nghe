use crate::{
    models::*,
    open_subsonic::common::{
        id3::{BasicArtistId3Record, SongId3},
        music_folder::check_user_music_folder_ids,
    },
    Database, DatabasePool, OSError,
};

use anyhow::Result;
use axum::extract::State;
use diesel::{dsl::sql, sql_types, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct GetSongParams {
    id: Uuid,
}

#[wrap_subsonic_response]
pub struct GetSongBody {
    song: SongId3,
}

async fn get_song(
    pool: &DatabasePool,
    music_folder_ids: &[Uuid],
    song_id: &Uuid,
) -> Result<SongId3> {
    songs::table
        .inner_join(songs_artists::table)
        .inner_join(artists::table.on(artists::id.eq(songs_artists::artist_id)))
        .filter(songs::music_folder_id.eq_any(music_folder_ids))
        .filter(songs::id.eq(song_id))
        .group_by(songs::id)
        .select((
            (
                songs::id,
                songs::title,
                songs::duration,
                songs::file_size,
                songs::created_at,
            ),
            sql::<sql_types::Array<BasicArtistId3Record>>(
                "array_agg(distinct(artists.id, artists.name)) basic_artists",
            ),
        ))
        .first::<SongId3>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
}

pub async fn get_song_handler(
    State(database): State<Database>,
    req: GetSongRequest,
) -> GetSongJsonResponse {
    let music_folder_ids = check_user_music_folder_ids(&database.pool, &req.user.id, None).await?;

    GetSongBody {
        song: get_song(&database.pool, &music_folder_ids, &req.params.id).await?,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::common::id3::BasicArtistId3,
        utils::{
            song::test::SongTag,
            test::{media::song_paths_to_ids, setup::setup_songs},
        },
    };

    use fake::{Fake, Faker};
    use itertools::Itertools;

    async fn get_basic_artists(
        pool: &DatabasePool,
        music_folder_ids: &[Uuid],
        song_id: &Uuid,
    ) -> Vec<BasicArtistId3> {
        songs::table
            .inner_join(albums::table)
            .inner_join(songs_artists::table)
            .inner_join(artists::table.on(artists::id.eq(songs_artists::artist_id)))
            .filter(songs::music_folder_id.eq_any(music_folder_ids))
            .filter(songs::id.eq(song_id))
            .select((artists::id, artists::name))
            .get_results::<BasicArtistId3>(&mut pool.get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .unique()
            .sorted()
            .collect_vec()
    }

    #[tokio::test]
    async fn test_get_song_id3() {
        let song_tag = Faker.fake::<SongTag>();

        let (temp_db, _temp_fs, music_folders, song_fs_infos) =
            setup_songs(&[1], vec![song_tag.clone()]).await;

        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let song_id = song_paths_to_ids(temp_db.pool(), &song_fs_infos)
            .await
            .remove(0);

        let song_id3 = get_song(temp_db.pool(), &music_folder_ids, &song_id)
            .await
            .unwrap();
        let basic_artists = get_basic_artists(temp_db.pool(), &music_folder_ids, &song_id).await;

        assert_eq!(song_id3.basic.title, song_tag.title);
        assert_eq!(
            song_id3.artists.into_iter().sorted().collect_vec(),
            basic_artists
        );
    }

    #[tokio::test]
    async fn test_get_song_id3_deny_music_folders() {
        let (temp_db, _temp_fs, music_folders, song_fs_infos) = setup_songs(&[1, 1], None).await;

        let music_folder_id = music_folders[0].id;
        let song_id = song_paths_to_ids(temp_db.pool(), &song_fs_infos[1..])
            .await
            .remove(0);

        assert!(matches!(
            get_song(temp_db.pool(), &[music_folder_id], &song_id)
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ));
    }
}
