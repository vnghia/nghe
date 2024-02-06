-- This file should undo anything in `up.sql`
drop function album_count_song_by_user (_user_id uuid, _album_id uuid);

drop function album_query_song_by_user (_user_id uuid, _album_id uuid);

drop function album_count_song_by_music_folder (_music_folder_ids uuid[], _album_id uuid);

drop function album_query_song_by_music_folder (_music_folder_ids uuid[], _album_id uuid);