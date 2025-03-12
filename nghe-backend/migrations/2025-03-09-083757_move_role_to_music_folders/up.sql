-- Your SQL goes here
alter table user_music_folder_permissions
add column owner boolean not null default false,
add column share boolean not null default false;

alter table users
drop column stream_role,
drop column download_role,
drop column share_role;

alter table users rename column admin_role to admin;
