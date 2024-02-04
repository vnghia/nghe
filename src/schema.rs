// @generated automatically by Diesel CLI.

diesel::table! {
    albums (id) {
        id -> Uuid,
        name -> Text,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
    }
}

diesel::table! {
    albums_artists (album_id, artist_id, song_id) {
        album_id -> Uuid,
        artist_id -> Uuid,
        song_id -> Uuid,
        upserted_at -> Timestamptz,
    }
}

diesel::table! {
    artists (id) {
        id -> Uuid,
        name -> Text,
        index -> Text,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
    }
}

diesel::table! {
    music_folders (id) {
        id -> Uuid,
        path -> Text,
        scanned_at -> Timestamptz,
    }
}

diesel::table! {
    songs (id) {
        id -> Uuid,
        title -> Text,
        album_id -> Uuid,
        music_folder_id -> Uuid,
        path -> Text,
        file_hash -> Int8,
        file_size -> Int8,
        updated_at -> Timestamptz,
        scanned_at -> Timestamptz,
    }
}

diesel::table! {
    songs_artists (song_id, artist_id) {
        song_id -> Uuid,
        artist_id -> Uuid,
        upserted_at -> Timestamptz,
    }
}

diesel::table! {
    user_music_folder_permissions (user_id, music_folder_id) {
        user_id -> Uuid,
        music_folder_id -> Uuid,
        allow -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        password -> Bytea,
        email -> Text,
        admin_role -> Bool,
        download_role -> Bool,
        share_role -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(albums_artists -> albums (album_id));
diesel::joinable!(albums_artists -> artists (artist_id));
diesel::joinable!(albums_artists -> songs (song_id));
diesel::joinable!(songs -> albums (album_id));
diesel::joinable!(songs -> music_folders (music_folder_id));
diesel::joinable!(songs_artists -> artists (artist_id));
diesel::joinable!(songs_artists -> songs (song_id));
diesel::joinable!(user_music_folder_permissions -> music_folders (music_folder_id));
diesel::joinable!(user_music_folder_permissions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    albums_artists,
    artists,
    music_folders,
    songs,
    songs_artists,
    user_music_folder_permissions,
    users,
);
