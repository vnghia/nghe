-- Your SQL goes here
drop index songs_music_folder_id_idx;

alter table songs drop column music_folder_id;

alter table albums
add column music_folder_id uuid not null,
add constraint albums_music_folder_id foreign key (music_folder_id) references music_folders (
    id
) on delete cascade;

create index albums_music_folder_id_idx on albums (music_folder_id);
