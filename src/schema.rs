// @generated automatically by Diesel CLI.

diesel::table! {
    albums (id) {
        id -> Uuid,
        name -> Text,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    artists (id) {
        id -> Uuid,
        name -> Text,
        index -> Text,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    music_folders (id) {
        id -> Uuid,
        path -> Text,
        updated_at -> Timestamptz,
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

diesel::joinable!(user_music_folder_permissions -> music_folders (music_folder_id));
diesel::joinable!(user_music_folder_permissions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    artists,
    music_folders,
    user_music_folder_permissions,
    users,
);
