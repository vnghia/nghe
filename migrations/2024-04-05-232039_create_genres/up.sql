-- Your SQL goes here
create table
genres (
    id uuid not null default gen_random_uuid() constraint genres_pkey primary key,
    value text not null,
    upserted_at timestamptz not null default now(),
    constraint genres_value_key unique (value)
);

create table
songs_genres (
    song_id uuid not null,
    genre_id uuid not null,
    upserted_at timestamptz not null default now(),
    constraint songs_genres_pkey primary key (song_id, genre_id),
    constraint songs_genres_song_id_fkey foreign key (
        song_id
    ) references songs (id) on delete cascade,
    constraint songs_genres_genre_id_fkey foreign key (
        genre_id
    ) references genres (id) on delete cascade
);
