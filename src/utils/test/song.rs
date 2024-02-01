use crate::models::*;
use crate::DatabasePool;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn query_all_song_information(
    pool: &DatabasePool,
    song_id: Uuid,
) -> (songs::Song, albums::Album, Vec<artists::Artist>) {
    let song = songs::table
        .filter(songs::id.eq(song_id))
        .select(songs::Song::as_select())
        .first(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song");

    let album = albums::table
        .filter(albums::id.eq(song.album_id))
        .select(albums::Album::as_select())
        .first(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query album");

    let artists = artists::table
        .inner_join(songs_artists::table)
        .filter(songs_artists::song_id.eq(song_id))
        .select(artists::Artist::as_select())
        .get_results(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song artists");

    (song, album, artists)
}

pub async fn query_all_songs_information(
    pool: &DatabasePool,
) -> HashMap<(Uuid, PathBuf), (songs::Song, albums::Album, Vec<artists::Artist>)> {
    let song_ids = songs::table
        .select(songs::id)
        .get_results(
            &mut pool
                .get()
                .await
                .expect("can not check out connection to the database"),
        )
        .await
        .expect("can not query song ids");
    stream::iter(song_ids)
        .then(|song_id| async move {
            let result = query_all_song_information(pool, song_id).await;
            (
                (result.0.music_folder_id, PathBuf::from(&result.0.path)),
                result,
            )
        })
        .collect::<HashMap<_, _>>()
        .await
}
