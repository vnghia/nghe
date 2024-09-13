-- This file should undo anything in `up.sql`
drop index songs_album_artists_song_id_idx;

drop index songs_album_artists_album_artist_id_idx;

drop table songs_album_artists;
