-- Your SQL goes here
-- fs type: 
--  1 - local
--  2 - s3
alter table music_folders
add column fs_type smallint not null default 1 constraint music_folders_fs_type check (
    fs_type between 1 and 2
);
