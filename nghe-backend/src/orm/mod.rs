pub mod albums;
pub mod artists;
pub mod binary;
pub mod configs;
pub mod function;
pub mod genres;
pub mod id3;
pub mod music_folders;
pub mod permission;
pub mod songs;
pub mod songs_album_artists;
pub mod songs_artists;
pub mod songs_genres;
pub mod upsert;
pub mod user_music_folder_permissions;
pub mod users;

pub type Type = diesel::pg::Pg;
