use std::path::Path;

use super::{album::upsert_album, artist::upsert_artists};
use crate::models::*;
use crate::utils::song::tag::SongTag;
use crate::{DatabasePool, OSResult};

use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

pub async fn upsert_song_artists(
    pool: &DatabasePool,
    song_id: Uuid,
    artist_ids: &[Uuid],
) -> OSResult<()> {
    diesel::insert_into(songs_artists::table)
        .values(
            artist_ids
                .iter()
                .cloned()
                .map(|artist_id| songs_artists::NewSongArtist { song_id, artist_id })
                .collect_vec(),
        )
        .on_conflict_do_nothing()
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_song<TI: AsRef<str>, TP: AsRef<Path>>(
    pool: &DatabasePool,
    ignored_prefixes: &[TI],
    music_folder_id: Uuid,
    song_id: Option<Uuid>,
    song_tag: SongTag,
    song_file_hash: u64,
    song_file_size: u64,
    song_relative_path: TP,
) -> OSResult<()> {
    let artist_ids = upsert_artists(pool, ignored_prefixes, &song_tag.artists).await?;
    let album_id = upsert_album(pool, song_tag.album.into()).await?;

    let song_id = if let Some(song_id) = song_id {
        let update_song = songs::UpdateSong {
            id: song_id,
            title: song_tag.title.into(),
            album_id,
            music_folder_id,
            file_hash: song_file_hash as i64,
            file_size: song_file_size as i64,
        };
        diesel::update(&update_song)
            .set(&update_song)
            .returning(songs::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?
    } else {
        let new_song = songs::NewSong {
            title: song_tag.title.into(),
            album_id,
            music_folder_id,
            path: song_relative_path.as_ref().to_string_lossy(),
            file_hash: song_file_hash as i64,
            file_size: song_file_size as i64,
        };
        diesel::insert_into(songs::table)
            .values(&new_song)
            .returning(songs::id)
            .get_result::<Uuid>(&mut pool.get().await?)
            .await?
    };

    upsert_song_artists(pool, song_id, &artist_ids).await?;

    Ok(())
}
