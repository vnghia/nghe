-- Your SQL goes here
alter table albums
add column cover_art_id uuid, add constraint cover_art_id_fkey foreign key (
    cover_art_id
) references cover_arts (id) on delete set null;
