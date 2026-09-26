-- Your SQL goes here
alter table artists
drop column lastfm_url,
drop column lastfm_mbz_id,
drop column lastfm_biography,
drop column cover_art_id,
drop column spotify_id;

create table
artist_informations (
    artist_id uuid not null constraint artist_informations_pkey primary key,
    lastfm_url text,
    lastfm_mbz_id uuid,
    lastfm_biography text,
    spotify_id text,
    cover_art_id uuid,
    constraint artist_informations_artist_id_fkey foreign key (
        artist_id
    ) references artists (id) on delete cascade,
    constraint artist_informations_cover_art_id_fkey foreign key (
        cover_art_id
    ) references cover_arts (id) on delete set null
);
