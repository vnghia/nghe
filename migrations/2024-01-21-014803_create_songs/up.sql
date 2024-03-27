-- Your SQL goes here
create table
  songs (
    id uuid not null default gen_random_uuid () constraint songs_pkey primary key,
    -- Song tag
    title text not null,
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
    languages text[] not null default array[]::text[] check (array_position(languages, null) is null),
    -- Song property
    format text not null,
    duration real not null,
    bitrate integer not null,
    sample_rate integer not null,
    channel_count smallint not null,
    -- Filesystem property
    music_folder_id uuid not null,
    relative_path text not null,
    file_hash bigint not null,
    file_size bigint not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    scanned_at timestamptz not null default now(),
    -- Constraints
    constraint songs_album_id_fkey foreign key (album_id) references albums (id) on delete cascade,
    constraint songs_music_folder_id_relative_path_key unique (music_folder_id, relative_path),
    constraint songs_music_folder_id_fkey foreign key (music_folder_id) references music_folders (id) on delete cascade,
    constraint songs_music_folder_id_file_hash_file_size_key unique (music_folder_id, file_hash, file_size)
  );

select
  add_updated_at_leave_scanned_at ('songs');

create index songs_album_id_idx on songs (album_id);

create index songs_music_folder_id_idx on songs (music_folder_id);