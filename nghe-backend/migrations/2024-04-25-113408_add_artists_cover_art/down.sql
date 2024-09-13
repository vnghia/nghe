-- This file should undo anything in `up.sql`
alter table artists
drop column spotify_id,
drop column cover_art_id;
