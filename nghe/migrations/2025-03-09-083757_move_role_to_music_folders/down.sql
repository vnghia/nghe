-- This file should undo anything in `up.sql`
alter table users rename column admin to admin_role;

alter table users
add column stream_role boolean not null default true,
add column download_role boolean not null default true,
add column share_role boolean not null default false;

alter table user_music_folder_permissions
drop column owner,
drop column share;
