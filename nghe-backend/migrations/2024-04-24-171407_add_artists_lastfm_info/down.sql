-- This file should undo anything in `up.sql`
alter table artists
drop column lastfm_url,
drop column lastfm_mbz_id,
drop column lastfm_biography;
