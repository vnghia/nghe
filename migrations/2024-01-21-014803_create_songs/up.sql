-- Your SQL goes here
create table
  songs (
    id uuid not null default gen_random_uuid () constraint songs_pkey primary key,
    title text not null,
    duration real not null,
    album_id uuid not null,
    track_number integer default null,
    track_total integer default null,
    disc_number integer default null,
    disc_total integer default null,
    year smallint default null,
    month smallint default null,
    day smallint default null,
    release_year smallint default null,
    release_month smallint default null,
    release_day smallint default null,
    original_release_year smallint default null,
    original_release_month smallint default null,
    original_release_day smallint default null,
    music_folder_id uuid not null,
    path text not null,
    file_hash bigint not null default 0,
    file_size bigint not null default 0,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    scanned_at timestamptz not null default now(),
    constraint songs_album_id_fkey foreign key (album_id) references albums (id) on delete cascade,
    constraint songs_music_folder_id_path_key unique (music_folder_id, path),
    constraint songs_music_folder_id_fkey foreign key (music_folder_id) references music_folders (id) on delete cascade,
    constraint songs_music_folder_id_file_hash_file_size_key unique (music_folder_id, file_hash, file_size)
  );

select
  add_updated_at_leave_scanned_at ('songs');

create index songs_album_id_idx on songs (album_id);

create index songs_music_folder_id_idx on songs (music_folder_id);