-- This file should undo anything in `up.sql`
drop index albums_artists_album_id_idx;

drop index albums_artists_artist_id_idx;

drop index albums_artists_song_id_idx;

drop table albums_artists;