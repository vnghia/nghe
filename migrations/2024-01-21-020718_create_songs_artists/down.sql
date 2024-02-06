-- This file should undo anything in `up.sql`
drop index songs_artists_song_id_idx;

drop index songs_artists_artist_id_idx;

drop table songs_artists;