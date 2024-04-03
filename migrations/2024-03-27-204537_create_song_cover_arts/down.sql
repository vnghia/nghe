-- This file should undo anything in `up.sql`
alter table songs drop column cover_art_id;

drop table song_cover_arts;
