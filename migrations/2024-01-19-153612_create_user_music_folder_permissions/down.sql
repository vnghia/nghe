-- This file should undo anything in `up.sql`
drop index user_music_folder_permissions_user_id_idx;

drop index user_music_folder_permissions_music_folder_id_idx;

drop table user_music_folder_permissions;
