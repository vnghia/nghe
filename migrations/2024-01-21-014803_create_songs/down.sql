-- This file should undo anything in `up.sql`
drop index songs_album_id_idx;

drop index songs_music_folder_id_idx;

drop table songs;