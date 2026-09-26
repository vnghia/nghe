-- This file should undo anything in `up.sql`
alter table user_music_folder_permissions add column allow boolean not null default true;
