-- Your SQL goes here
alter table scans
add column music_folder_id uuid not null,
add constraint scans_music_folder_id foreign key (music_folder_id) references music_folders (
    id
) on delete cascade;

alter table scans
drop constraint scans_pkey,
add constraint scans_pkey primary key (started_at, music_folder_id);

alter table scans drop column error_message, drop column scanned_count;

alter table scans add column scanned_song_count bigint not null default 0;
alter table scans add column upserted_song_count bigint not null default 0;
alter table scans add column deleted_song_count bigint not null default 0;
alter table scans add column deleted_album_count bigint not null default 0;
alter table scans add column deleted_artist_count bigint not null default 0;
alter table scans add column deleted_genre_count bigint not null default 0;
alter table scans add column scan_error_count bigint not null default 0;

alter table scans add column unrecoverable bool,
drop constraint scans_not_scanning_if_finished,
add constraint scans_not_scanning_if_finished_and_set_unrecoverable check (
    (
        is_scanning
        and unrecoverable is null
        and finished_at is null
    )
    or (
        not is_scanning
        and unrecoverable is not null
        and finished_at is not null
    )
);

drop index scans_is_scanning_idx;
create unique index scans_music_folder_id_is_scanning_idx on scans (music_folder_id, is_scanning)
where is_scanning;
