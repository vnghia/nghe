-- Your SQL goes here
create table
  songs (
    id uuid not null default gen_random_uuid () constraint songs_pkey primary key,
    title text not null,
    album_id uuid not null,
    music_folder_id uuid not null,
    path text not null,
    file_hash bigint not null default 0,
    file_size bigint not null default 0,
    updated_at timestamptz not null default now(),
    scanned_at timestamptz not null default now(),
    constraint songs_album_id_fkey foreign key (album_id) references albums (id) on delete cascade,
    constraint songs_music_folder_id_path_key unique (music_folder_id, path),
    constraint songs_music_folder_id_fkey foreign key (music_folder_id) references music_folders (id) on delete cascade,
    constraint songs_hash_size_key unique (file_hash, file_size)
  );

select
  add_updated_at_leave_scanned_at ('songs');