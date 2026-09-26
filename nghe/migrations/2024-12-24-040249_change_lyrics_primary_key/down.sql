-- This file should undo anything in `up.sql`
drop table lyrics;

create table
lyrics (
    song_id uuid not null,
    description text not null,
    language text not null,
    line_values text [] not null check (
        array_position(line_values, null) is null
    ),
    line_starts integer [] check (
        array_position(line_starts, null) is null
    ),
    lyric_hash bigint not null,
    lyric_size integer not null,
    external bool not null,
    updated_at timestamptz not null default now(),
    scanned_at timestamptz not null default now(),
    check (line_starts is null or array_length(line_values, 1) = array_length(line_starts, 1)),
    constraint lyrics_song_id_key foreign key (song_id) references songs (
        id
    ) on delete cascade,
    constraint lyrics_pkey primary key (
        song_id, description, language, external
    )
);

select add_updated_at_leave_scanned_at('lyrics');
