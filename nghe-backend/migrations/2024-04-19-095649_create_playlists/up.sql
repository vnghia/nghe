-- Your SQL goes here
create table playlists (
    id uuid not null default gen_random_uuid() constraint playlists_pkey primary key,
    name text not null,
    comment text default null,
    public boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

select add_updated_at('playlists');

-- access level: 
--  1 - read songs
--  2 - add/remove songs
--  3 - admin (add/remove users, edit, delete)
create table playlists_users (
    playlist_id uuid not null,
    user_id uuid not null,
    access_level smallint not null constraint playlists_users_access_level check (
        access_level between 1 and 3
    ),
    constraint playlists_users_pkey primary key (playlist_id, user_id),
    constraint playlists_users_playlist_id_fkey foreign key (
        playlist_id
    ) references playlists (id) on delete cascade,
    constraint playlists_users_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade
);

create table playlists_songs (
    playlist_id uuid not null,
    song_id uuid not null,
    created_at timestamptz not null default now(),
    constraint playlists_songs_pkey primary key (playlist_id, song_id),
    constraint playlists_songs_playlist_id_fkey foreign key (
        playlist_id
    ) references playlists (id) on delete cascade,
    constraint playlists_songs_song_id_fkey foreign key (
        song_id
    ) references songs (id) on delete cascade
)
