-- This file should undo anything in `up.sql`
drop index albums_music_folder_id_name_date_release_original_idx;

create unique index albums_name_date_release_original_idx on albums (
    name,
    year,
    month,
    day,
    release_year,
    release_month,
    release_day,
    original_release_year,
    original_release_month,
    original_release_day
) nulls not distinct where (mbz_id is null);

alter table albums
drop constraint albums_music_folder_id_mbz_id_key,
add constraint albums_mbz_id_key unique (mbz_id);

drop index albums_music_folder_id_idx;

alter table albums drop column music_folder_id;

alter table songs
add column music_folder_id uuid not null,
add constraint songs_music_folder_id foreign key (music_folder_id) references music_folders (
    id
) on delete cascade;

create index songs_music_folder_id_idx on songs (music_folder_id);
