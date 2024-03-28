-- Your SQL goes here
create table
song_cover_arts (
    id uuid not null default gen_random_uuid() constraint song_cover_arts_pkey primary key,
    format text not null,
    file_hash bigint not null,
    file_size bigint not null,
    upserted_at timestamptz not null default now(),
    constraint song_cover_arts_format_file_hash_file_size_key unique (
        format, file_hash, file_size
    )
);

alter table songs
add cover_art_id uuid, add constraint songs_cover_art_id_fkey foreign key (
    cover_art_id
) references song_cover_arts (id) on delete set null;
