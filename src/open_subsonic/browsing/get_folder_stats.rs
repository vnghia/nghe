use anyhow::Result;
use axum::extract::State;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl::{count_distinct, sum, AssumeNotNull};
use diesel::{
    helper_types, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, Queryable,
    Selectable, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetFolderStatsParams, admin);
add_axum_response!(GetFolderStatsBody);

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct FolderStats {
    #[diesel(embed)]
    music_folder: music_folders::MusicFolder,
    #[diesel(select_expression = count_distinct(artists::id))]
    #[diesel(select_expression_type = count_distinct<artists::id>)]
    artist_count: i64,
    #[diesel(select_expression = count_distinct(songs::album_id))]
    #[diesel(select_expression_type = count_distinct<songs::album_id>)]
    album_count: i64,
    #[diesel(select_expression = count_distinct(songs::id))]
    #[diesel(select_expression_type = count_distinct<songs::id>)]
    song_count: i64,
    #[diesel(select_expression = count_distinct(user_music_folder_permissions::user_id))]
    #[diesel(select_expression_type = count_distinct<user_music_folder_permissions::user_id>)]
    user_count: i64,
    #[diesel(select_expression = sum(songs::file_size).assume_not_null())]
    #[diesel(select_expression_type = AssumeNotNull<helper_types::sum<songs::file_size>>)]
    total_size: BigDecimal,
}

async fn get_folder_stats(pool: &DatabasePool) -> Result<Vec<FolderStats>> {
    get_basic_artist_id3_db()
        .inner_join(music_folders::table.on(music_folders::id.eq(songs::music_folder_id)))
        .inner_join(
            user_music_folder_permissions::table
                .on(user_music_folder_permissions::music_folder_id.eq(music_folders::id)),
        )
        .group_by(music_folders::id)
        .select(FolderStats::as_select())
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_folder_stats_handler(
    State(database): State<Database>,
    _: GetFolderStatsRequest,
) -> GetFolderStatsJsonResponse {
    Ok(axum::Json(
        GetFolderStatsBody {
            folder_stats: get_folder_stats(&database.pool)
                .await?
                .into_iter()
                .map(FolderStats::into)
                .collect(),
        }
        .into(),
    ))
}

impl From<FolderStats> for nghe_types::browsing::get_folder_stats::FolderStats {
    fn from(value: FolderStats) -> Self {
        Self {
            music_folder: value.music_folder.into(),
            artist_count: value.artist_count as _,
            album_count: value.album_count as _,
            song_count: value.song_count as _,
            user_count: value.user_count as _,
            total_size: value.total_size.to_usize().expect("can not convert total size to usize"),
        }
    }
}
