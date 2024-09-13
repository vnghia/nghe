-- This file should undo anything in `up.sql`
drop index albums_music_folder_id_idx;

alter table albums drop column music_folder_id;

alter table songs
add column music_folder_id uuid not null,
add constraint songs_music_folder_id foreign key (music_folder_id) references music_folders (
    id
) on delete cascade;

create index songs_music_folder_id_idx on songs (music_folder_id);
