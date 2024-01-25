-- Your SQL goes here
create table
  songs_artists (
    song_id uuid not null,
    artist_id uuid not null,
    constraint songs_artists_pkey primary key (song_id, artist_id),
    constraint songs_artists_song_id_fkey foreign key (song_id) references songs (id) on delete cascade,
    constraint songs_artists_artist_id_fkey foreign key (artist_id) references artists (id) on delete cascade
  );