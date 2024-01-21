-- Your SQL goes here
create table
  songs (
    id uuid not null default gen_random_uuid () constraint songs_pkey primary key,
    title text not null,
    album_id uuid not null,
    music_folder_id uuid not null,
    path text not null,
    constraint songs_album_id_fkey foreign key (album_id) references albums (id),
    constraint songs_music_folder_id_path_key unique (music_folder_id, path),
    constraint songs_music_folder_id_fkey foreign key (music_folder_id) references music_folders (id)
  )