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

    genres (id) {
        id -> Uuid,
        value -> Text,
        upserted_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    lyrics (song_id, description, language, external) {
        song_id -> Uuid,
        description -> Text,
        language -> Text,
        line_values -> Array<Nullable<Text>>,
        line_starts -> Nullable<Array<Nullable<Int4>>>,
        lyric_hash -> Int8,
        lyric_size -> Int4,
        external -> Bool,
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

    song_cover_arts (id) {
        id -> Uuid,
        format -> Text,
        file_hash -> Int8,
        file_size -> Int4,
        upserted_at -> Timestamptz,
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
        music_folder_id -> Uuid,
        relative_path -> Text,
        file_hash -> Int8,
        file_size -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
        cover_art_id -> Nullable<Uuid>,
        mbz_id -> Nullable<Uuid>,
        ts -> Tsvector,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_full_text_search::*;

    songs_album_artists (song_id, album_artist_id) {
        song_id -> Uuid,
        album_artist_id -> Uuid,
        upserted_at -> Timestamptz,
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

    user_music_folder_permissions (user_id, music_folder_id) {
        user_id -> Uuid,
        music_folder_id -> Uuid,
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
        admin_role -> Bool,
        stream_role -> Bool,
        download_role -> Bool,
        share_role -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(lyrics -> songs (song_id));
diesel::joinable!(playbacks -> songs (song_id));
diesel::joinable!(playbacks -> users (user_id));
diesel::joinable!(scans -> music_folders (music_folder_id));
diesel::joinable!(songs -> albums (album_id));
diesel::joinable!(songs -> music_folders (music_folder_id));
diesel::joinable!(songs -> song_cover_arts (cover_art_id));
diesel::joinable!(songs_album_artists -> artists (album_artist_id));
diesel::joinable!(songs_album_artists -> songs (song_id));
diesel::joinable!(songs_artists -> artists (artist_id));
diesel::joinable!(songs_artists -> songs (song_id));
diesel::joinable!(songs_genres -> genres (genre_id));
diesel::joinable!(songs_genres -> songs (song_id));
diesel::joinable!(user_music_folder_permissions -> music_folders (music_folder_id));
diesel::joinable!(user_music_folder_permissions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    artists,
    configs,
    genres,
    lyrics,
    music_folders,
    playbacks,
    scans,
    song_cover_arts,
    songs,
    songs_album_artists,
    songs_artists,
    songs_genres,
    user_music_folder_permissions,
    users,
);
