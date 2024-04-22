use anyhow::Result;
use axum::extract::State;
use diesel::dsl::{count_distinct, sum, AssumeNotNull, Eq, Filter, Nullable, Select, SingleValue};
use diesel::query_source::{Alias, AliasedField};
use diesel::{
    helper_types, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    QueryDsl, Queryable, Selectable, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate, add_convert_types};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::sql::coalesce;
use crate::open_subsonic::sql::coalesce::HelperType as Coalesce;
use crate::{Database, DatabasePool};

add_common_validate!(GetMusicFolderStatParams, admin);
add_axum_response!(GetMusicFolderStatBody);

diesel::alias!(songs as songs_total_size: SongsTotalSize);

#[add_convert_types(into = nghe_types::music_folder::get_music_folder_stat::MusicFolderStat)]
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct MusicFolderStatDb {
    #[diesel(embed)]
    pub music_folder: music_folders::MusicFolder,
    #[diesel(select_expression = count_distinct(artists::id.nullable()))]
    #[diesel(select_expression_type = count_distinct<Nullable<artists::id>>)]
    pub artist_count: i64,
    #[diesel(select_expression = count_distinct(songs::album_id.nullable()))]
    #[diesel(select_expression_type = count_distinct<Nullable<songs::album_id>>)]
    pub album_count: i64,
    #[diesel(select_expression = count_distinct(songs::id.nullable()))]
    #[diesel(select_expression_type = count_distinct<Nullable<songs::id>>)]
    pub song_count: i64,
    #[diesel(select_expression = count_distinct(user_music_folder_permissions::user_id.nullable()))]
    #[diesel(select_expression_type =
        count_distinct<Nullable<user_music_folder_permissions::user_id>>
    )]
    pub user_count: i64,
    #[diesel(select_expression = songs_total_size
        .filter(songs_total_size.field(songs::music_folder_id).eq(music_folders::id))
        .select(coalesce(sum(songs_total_size.field(songs::file_size)), 0))
        .single_value()
        .assume_not_null()
    )]
    #[diesel(select_expression_type = AssumeNotNull<
        SingleValue<
            Select<
                Filter<
                    Alias<SongsTotalSize>,
                    Eq<AliasedField<SongsTotalSize, songs::music_folder_id>, music_folders::id>,
                >,
                Coalesce<helper_types::sum<AliasedField<SongsTotalSize, songs::file_size>>, i64>,
            >,
        >,
    >)]
    pub total_size: i64,
}

async fn get_music_folder_stat(pool: &DatabasePool, id: Uuid) -> Result<MusicFolderStatDb> {
    music_folders::table
        .left_join(songs::table)
        .left_join(songs_album_artists::table.on(songs_album_artists::song_id.eq(songs::id)))
        .left_join(songs_artists::table.on(songs_artists::song_id.eq(songs::id)))
        .left_join(
            artists::table.on(artists::id
                .eq(songs_album_artists::album_artist_id)
                .or(artists::id.eq(songs_artists::artist_id))),
        )
        .left_join(user_music_folder_permissions::table)
        .filter(music_folders::id.eq(id))
        .group_by(music_folders::id)
        .select(MusicFolderStatDb::as_select())
        .get_result(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_music_folder_stat_handler(
    State(database): State<Database>,
    req: GetMusicFolderStatRequest,
) -> GetMusicFolderStatJsonResponse {
    Ok(axum::Json(
        GetMusicFolderStatBody {
            stat: get_music_folder_stat(&database.pool, req.params.id).await?.into(),
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rand::prelude::SliceRandom;

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_folder_stat_empty() {
        let infra = Infra::new().await.add_user(None).await.add_folder(false).await;
        let stat = get_music_folder_stat(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert_eq!(
            MusicFolderStatDb {
                music_folder: infra.music_folders[0].clone(),
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                user_count: 0,
                total_size: 0,
            },
            stat
        );
    }

    #[tokio::test]
    async fn test_get_folder_stat_empty_with_user() {
        let infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        let stat = get_music_folder_stat(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert_eq!(
            MusicFolderStatDb {
                music_folder: infra.music_folders[0].clone(),
                artist_count: 0,
                album_count: 0,
                song_count: 0,
                user_count: 1,
                total_size: 0,
            },
            stat
        );
    }

    #[tokio::test]
    async fn test_get_folder_stat() {
        let n_song = (10..20).fake();
        let artists = fake::vec![String; 2..5];
        let album = fake::vec![String; 2..5];

        let mut thread_rng = rand::thread_rng();

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag {
                        artists: artists
                            .choose_multiple(&mut thread_rng, (1..2).fake())
                            .cloned()
                            .map(String::into)
                            .collect(),
                        album_artists: artists
                            .choose_multiple(&mut thread_rng, (1..2).fake())
                            .cloned()
                            .map(String::into)
                            .collect(),
                        album: album.choose(&mut thread_rng).unwrap().to_string().into(),
                        ..Faker.fake()
                    })
                    .collect(),
            )
            .scan(.., None)
            .await;

        let artist_count = infra.artist_no_ids(..).len() as _;
        let album_count = infra.album_no_ids(..).len() as _;

        let song_fs_infos = infra.song_fs_infos(..);
        let song_count = song_fs_infos.len() as _;
        let total_size = song_fs_infos.iter().fold(0_u32, |aac, s| aac + s.file_size) as _;

        let stat = get_music_folder_stat(infra.pool(), infra.music_folder_id(0)).await.unwrap();
        assert_eq!(
            MusicFolderStatDb {
                music_folder: infra.music_folders[0].clone(),
                artist_count,
                album_count,
                song_count,
                user_count: 1,
                total_size,
            },
            stat
        );
    }
}
