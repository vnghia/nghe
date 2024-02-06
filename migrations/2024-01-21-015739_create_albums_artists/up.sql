-- Your SQL goes here
create table
  albums_artists (
    album_id uuid not null,
    artist_id uuid not null,
    song_id uuid not null,
    upserted_at timestamptz not null default now(),
    constraint albums_artists_pkey primary key (album_id, artist_id, song_id),
    constraint albums_artists_album_id_fkey foreign key (album_id) references albums (id) on delete cascade,
    constraint albums_artists_artist_id_fkey foreign key (artist_id) references artists (id) on delete cascade,
    constraint albums_artists_song_id_fkey foreign key (song_id) references songs (id) on delete cascade
  );

create index albums_artists_album_id_idx on albums_artists (album_id);

create index albums_artists_artist_id_idx on albums_artists (artist_id);

create index albums_artists_song_id_idx on albums_artists (song_id);