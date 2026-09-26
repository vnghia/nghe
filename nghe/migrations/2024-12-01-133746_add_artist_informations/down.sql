-- This file should undo anything in `up.sql`
drop table artist_informations;

alter table artists
add column lastfm_url text,
add column lastfm_mbz_id uuid,
add column lastfm_biography text,
add column cover_art_id uuid,
add column spotify_id text,
add constraint cover_art_id_fkey foreign key (
    cover_art_id
) references cover_arts (id) on delete set null;
