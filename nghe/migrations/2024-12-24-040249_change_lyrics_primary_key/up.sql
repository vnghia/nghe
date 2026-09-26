-- Your SQL goes here
drop table lyrics;

create table
lyrics (
    id uuid not null default gen_random_uuid() constraint lyrics_pkey primary key,
    song_id uuid not null,
    external boolean not null,
    description text,
    language text not null,
    durations integer [] check (array_position(durations, null) is null),
    texts text [] not null check (array_position(texts, null) is null),
    updated_at timestamptz not null default now(),
    scanned_at timestamptz not null default now(),
    check (durations is null or array_length(durations, 1) = array_length(texts, 1)),
    constraint lyrics_song_id_fkey foreign key (song_id) references songs (
        id
    ) on delete cascade
);

select add_updated_at_leave_scanned_at('lyrics');

create unique index lyrics_song_id_key on lyrics (
    song_id
) nulls not distinct where (external);

create unique index lyrics_song_id_description_key on lyrics (
    song_id, description
) nulls not distinct where (not external);

create index lyrics_song_id_idx on lyrics (song_id);
