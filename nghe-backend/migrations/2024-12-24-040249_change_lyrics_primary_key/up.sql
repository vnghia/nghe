-- Your SQL goes here
alter table lyrics
drop column lyric_hash,
drop column lyric_size,
drop constraint lyrics_pkey,
add constraint lyrics_pkey primary key (song_id, description, external);
