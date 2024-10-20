-- Your SQL goes here
alter table music_folders
add column created_at timestamptz not null default now();
