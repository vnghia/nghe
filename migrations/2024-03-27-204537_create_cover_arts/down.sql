-- This file should undo anything in `up.sql`
alter table songs drop column cover_art_id;

drop table cover_arts;
