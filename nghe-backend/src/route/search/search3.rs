#![allow(clippy::too_many_lines)]

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use diesel_full_text_search::configuration::TsConfigurationByName;
use diesel_full_text_search::{
    ts_rank_cd, websearch_to_tsquery_with_search_config, TsVectorExtensions,
};
use nghe_api::search::search3::SearchResult3;
pub use nghe_api::search::search3::{Request, Response};
use nghe_proc_macro::{check_music_folder, handler};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::{albums, artists, id3, songs};
use crate::Error;

const USIMPLE_TS_CONFIGURATION: TsConfigurationByName = TsConfigurationByName("usimple");

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let search_query = &request.query;
    let sync = search_query.is_empty() || search_query == "\"\"";

    #[check_music_folder]
    {
        let count = request.artist_count.unwrap_or(20).into();
        let artist = if count > 0 {
            let offset = request.artist_offset.unwrap_or(0).into();
            let query = id3::artist::query::with_user_id(user_id).limit(count).offset(offset);
            if sync {
                query
                    .order_by((artists::name, artists::mbz_id))
                    .get_results(&mut database.get().await?)
                    .await?
            } else {
                query
                    .filter(artists::ts.matches(websearch_to_tsquery_with_search_config(
                        USIMPLE_TS_CONFIGURATION,
                        search_query,
                    )))
                    .order_by(
                        ts_rank_cd(
                            artists::ts,
                            websearch_to_tsquery_with_search_config(
                                USIMPLE_TS_CONFIGURATION,
                                search_query,
                            ),
                        )
                        .desc(),
                    )
                    .get_results(&mut database.get().await?)
                    .await?
            }
        } else {
            vec![]
        };

        let count = request.album_count.unwrap_or(20).into();
        let album = if count > 0 {
            let offset = request.album_offset.unwrap_or(0).into();
            let query = id3::album::short::query::with_user_id(user_id).limit(count).offset(offset);
            if sync {
                query
                    .order_by((albums::name, albums::mbz_id))
                    .get_results(&mut database.get().await?)
                    .await?
            } else {
                query
                    .filter(albums::ts.matches(websearch_to_tsquery_with_search_config(
                        USIMPLE_TS_CONFIGURATION,
                        search_query,
                    )))
                    .order_by(
                        ts_rank_cd(
                            albums::ts,
                            websearch_to_tsquery_with_search_config(
                                USIMPLE_TS_CONFIGURATION,
                                search_query,
                            ),
                        )
                        .desc(),
                    )
                    .get_results(&mut database.get().await?)
                    .await?
            }
        } else {
            vec![]
        };

        let count = request.song_count.unwrap_or(20).into();
        let song = if count > 0 {
            let offset = request.song_offset.unwrap_or(0).into();
            let query = id3::song::short::query::with_user_id(user_id).limit(count).offset(offset);
            if sync {
                query
                    .order_by((songs::title, songs::mbz_id))
                    .get_results(&mut database.get().await?)
                    .await?
            } else {
                query
                    .filter(songs::ts.matches(websearch_to_tsquery_with_search_config(
                        USIMPLE_TS_CONFIGURATION,
                        search_query,
                    )))
                    .order_by(
                        ts_rank_cd(
                            songs::ts,
                            websearch_to_tsquery_with_search_config(
                                USIMPLE_TS_CONFIGURATION,
                                search_query,
                            ),
                        )
                        .desc(),
                    )
                    .get_results(&mut database.get().await?)
                    .await?
            }
        } else {
            vec![]
        };

        Ok(Response {
            search_result3: SearchResult3 {
                artist: artist.into_iter().map(id3::artist::Artist::try_into).try_collect()?,
                album: album.into_iter().map(id3::album::short::Short::try_into).try_collect()?,
                song: song.into_iter().map(id3::song::short::Short::try_into).try_collect()?,
            },
        })
    }
}
