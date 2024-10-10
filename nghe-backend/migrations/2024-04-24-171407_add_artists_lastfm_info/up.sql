-- Your SQL goes here
alter table artists
add column lastfm_url text,
add column lastfm_mbz_id uuid,
add column lastfm_biography text;
