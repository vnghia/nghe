-- Your SQL goes here
create extension unaccent schema public;

create text search configuration usimple (copy = simple);
alter text search configuration usimple
alter mapping for hword, hword_part, word with public.unaccent, simple;

alter table artists
add column ts tsvector not null generated always as (to_tsvector('usimple', name)) stored;
create index artists_ts_idx on artists using gin (ts);

alter table albums
add column ts tsvector not null generated always as (to_tsvector('usimple', name)) stored;
create index albums_ts_idx on albums using gin (ts);

alter table songs
add column ts tsvector not null generated always as (to_tsvector('usimple', title)) stored;
create index songs_ts_idx on songs using gin (ts);
