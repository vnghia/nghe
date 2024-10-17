-- This file should undo anything in `up.sql`
create unique index scans_is_scanning_idx on scans (is_scanning)
where is_scanning;
drop index scans_music_folder_id_is_scanning_idx;

alter table scans drop column unrecoverable,
add constraint scans_not_scanning_if_finished check (
    (
        is_scanning
        and finished_at is null
    )
    or (
        not is_scanning
        and finished_at is not null
    )
);

alter table scans drop column scanned_song_count;
alter table scans drop column upserted_song_count;
alter table scans drop column deleted_song_count;
alter table scans drop column deleted_album_count;
alter table scans drop column deleted_artist_count;
alter table scans drop column deleted_genre_count;
alter table scans drop column scan_error_count;

alter table scans add column error_message text,
add column scanned_count bigint not null default 0;

alter table scans
drop constraint scans_pkey,
add constraint scans_pkey primary key (started_at);

alter table scans drop column music_folder_id;
