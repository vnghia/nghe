-- Your SQL goes here
create table star_artists (
    user_id uuid not null,
    artist_id uuid not null,
    created_at timestamptz not null default now(),
    constraint star_artists_pkey primary key (user_id, artist_id),
    constraint star_artists_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade,
    constraint star_artists_album_id_fkey foreign key (
        artist_id
    ) references artists (id) on delete cascade
);

create table star_albums (
    user_id uuid not null,
    album_id uuid not null,
    created_at timestamptz not null default now(),
    constraint star_albums_pkey primary key (user_id, album_id),
    constraint star_albums_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade,
    constraint star_albums_album_id_fkey foreign key (
        album_id
    ) references albums (id) on delete cascade
);

create table star_songs (
    user_id uuid not null,
    song_id uuid not null,
    created_at timestamptz not null default now(),
    constraint star_songs_pkey primary key (user_id, song_id),
    constraint star_songs_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade,
    constraint star_songs_song_id_fkey foreign key (
        song_id
    ) references songs (id) on delete cascade
);
