-- This file should undo anything in `up.sql`
alter table lyrics
add column lyric_hash bigint not null,
add column lyric_size integer not null,
drop constraint lyrics_pkey,
add constraint lyrics_pkey primary key (song_id, description, language, external);
