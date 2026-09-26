-- Your SQL goes here
alter table music_folders add name text not null,
add constraint music_folders_name_key unique (name);

alter table music_folders
add updated_at timestamptz not null default now();

select add_updated_at_leave_scanned_at('music_folders');
