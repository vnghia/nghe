// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    albums (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
        year -> Nullable<Int2>,
        month -> Nullable<Int2>,
        day -> Nullable<Int2>,
        release_year -> Nullable<Int2>,
        release_month -> Nullable<Int2>,
        release_day -> Nullable<Int2>,
        original_release_year -> Nullable<Int2>,
        original_release_month -> Nullable<Int2>,
        original_release_day -> Nullable<Int2>,
        mbz_id -> Nullable<Uuid>,
        ts -> Tsvector,
        music_folder_id -> Uuid,
        cover_art_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    artist_informations (artist_id) {
        artist_id -> Uuid,
        lastfm_url -> Nullable<Text>,
        lastfm_mbz_id -> Nullable<Uuid>,
        lastfm_biography -> Nullable<Text>,
        spotify_id -> Nullable<Text>,
        cover_art_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    artists (id) {
        id -> Uuid,
        name -> Text,
        index -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
        mbz_id -> Nullable<Uuid>,
        ts -> Tsvector,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    configs (key) {
        key -> Text,
        text -> Nullable<Text>,
        byte -> Nullable<Bytea>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    cover_arts (id) {
        id -> Uuid,
        format -> Text,
        file_hash -> Int8,
        file_size -> Int4,
        source -> Nullable<Text>,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    genres (id) {
        id -> Uuid,
        value -> Text,
        upserted_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    lyrics (id) {
        id -> Uuid,
        song_id -> Uuid,
        external -> Bool,
        description -> Nullable<Text>,
        language -> Text,
        durations -> Nullable<Array<Nullable<Int4>>>,
        texts -> Array<Nullable<Text>>,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    music_folders (id) {
        id -> Uuid,
        path -> Text,
        scanned_at -> Timestamptz,
        name -> Text,
        updated_at -> Timestamptz,
        fs_type -> Int2,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    playbacks (user_id, song_id) {
        user_id -> Uuid,
        song_id -> Uuid,
        count -> Int4,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    playlists (id) {
        id -> Uuid,
        name -> Text,
        comment -> Nullable<Text>,
        public -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    playlists_songs (playlist_id, song_id) {
        playlist_id -> Uuid,
        song_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    playlists_users (playlist_id, user_id) {
        playlist_id -> Uuid,
        user_id -> Uuid,
        write -> Bool,
        owner -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    playqueues (user_id) {
        user_id -> Uuid,
        ids -> Array<Nullable<Uuid>>,
        current -> Nullable<Uuid>,
        position -> Nullable<Int8>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    scans (started_at, music_folder_id) {
        started_at -> Timestamptz,
        is_scanning -> Bool,
        finished_at -> Nullable<Timestamptz>,
        music_folder_id -> Uuid,
        scanned_song_count -> Int8,
        upserted_song_count -> Int8,
        deleted_song_count -> Int8,
        deleted_album_count -> Int8,
        deleted_artist_count -> Int8,
        deleted_genre_count -> Int8,
        scan_error_count -> Int8,
        unrecoverable -> Nullable<Bool>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    songs (id) {
        id -> Uuid,
        title -> Text,
        album_id -> Uuid,
        track_number -> Nullable<Int4>,
        track_total -> Nullable<Int4>,
        disc_number -> Nullable<Int4>,
        disc_total -> Nullable<Int4>,
        year -> Nullable<Int2>,
        month -> Nullable<Int2>,
        day -> Nullable<Int2>,
        release_year -> Nullable<Int2>,
        release_month -> Nullable<Int2>,
        release_day -> Nullable<Int2>,
        original_release_year -> Nullable<Int2>,
        original_release_month -> Nullable<Int2>,
        original_release_day -> Nullable<Int2>,
        languages -> Array<Nullable<Text>>,
        format -> Text,
        duration -> Float4,
        bitrate -> Int4,
        sample_rate -> Int4,
        channel_count -> Int2,
        relative_path -> Text,
        file_hash -> Int8,
        file_size -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
        cover_art_id -> Nullable<Uuid>,
        mbz_id -> Nullable<Uuid>,
        ts -> Tsvector,
        bit_depth -> Nullable<Int2>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    songs_album_artists (song_id, album_artist_id) {
        song_id -> Uuid,
        album_artist_id -> Uuid,
        upserted_at -> Timestamptz,
        compilation -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    songs_artists (song_id, artist_id) {
        song_id -> Uuid,
        artist_id -> Uuid,
        upserted_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    songs_genres (song_id, genre_id) {
        song_id -> Uuid,
        genre_id -> Uuid,
        upserted_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    star_albums (user_id, album_id) {
        user_id -> Uuid,
        album_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    star_artists (user_id, artist_id) {
        user_id -> Uuid,
        artist_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    star_songs (user_id, song_id) {
        user_id -> Uuid,
        song_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    user_keys (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    user_music_folder_permissions (user_id, music_folder_id) {
        user_id -> Uuid,
        music_folder_id -> Uuid,
        owner -> Bool,
        share -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    users (id) {
        id -> Uuid,
        username -> Text,
        password -> Bytea,
        email -> Text,
        admin -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(albums -> cover_arts (cover_art_id));
diesel::joinable!(albums -> music_folders (music_folder_id));
diesel::joinable!(artist_informations -> artists (artist_id));
diesel::joinable!(artist_informations -> cover_arts (cover_art_id));
diesel::joinable!(lyrics -> songs (song_id));
diesel::joinable!(playbacks -> songs (song_id));
diesel::joinable!(playbacks -> users (user_id));
diesel::joinable!(playlists_songs -> playlists (playlist_id));
diesel::joinable!(playlists_songs -> songs (song_id));
diesel::joinable!(playlists_users -> playlists (playlist_id));
diesel::joinable!(playlists_users -> users (user_id));
diesel::joinable!(playqueues -> users (user_id));
diesel::joinable!(scans -> music_folders (music_folder_id));
diesel::joinable!(songs -> albums (album_id));
diesel::joinable!(songs -> cover_arts (cover_art_id));
diesel::joinable!(songs_album_artists -> artists (album_artist_id));
diesel::joinable!(songs_album_artists -> songs (song_id));
diesel::joinable!(songs_artists -> artists (artist_id));
diesel::joinable!(songs_artists -> songs (song_id));
diesel::joinable!(songs_genres -> genres (genre_id));
diesel::joinable!(songs_genres -> songs (song_id));
diesel::joinable!(star_albums -> albums (album_id));
diesel::joinable!(star_albums -> users (user_id));
diesel::joinable!(star_artists -> artists (artist_id));
diesel::joinable!(star_artists -> users (user_id));
diesel::joinable!(star_songs -> songs (song_id));
diesel::joinable!(star_songs -> users (user_id));
diesel::joinable!(user_keys -> users (user_id));
diesel::joinable!(user_music_folder_permissions -> music_folders (music_folder_id));
diesel::joinable!(user_music_folder_permissions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    artist_informations,
    artists,
    configs,
    cover_arts,
    genres,
    lyrics,
    music_folders,
    playbacks,
    playlists,
    playlists_songs,
    playlists_users,
    playqueues,
    scans,
    songs,
    songs_album_artists,
    songs_artists,
    songs_genres,
    star_albums,
    star_artists,
    star_songs,
    user_keys,
    user_music_folder_permissions,
    users,
);
