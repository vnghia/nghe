-- Your SQL goes here
create table
songs_album_artists (
    song_id uuid not null,
    album_artist_id uuid not null,
    upserted_at timestamptz not null default now(),
    constraint songs_album_artists_pkey primary key (song_id, album_artist_id),
    constraint songs_album_artists_song_id_fkey foreign key (
        song_id
    ) references songs (id) on delete cascade,
    constraint songs_album_artists_album_artist_id_fkey foreign key (
        album_artist_id
    ) references artists (id) on delete cascade
);

create index songs_album_artists_song_id_idx on songs_album_artists (song_id);

create index songs_album_artists_album_artist_id_idx on songs_album_artists (
    album_artist_id
);
