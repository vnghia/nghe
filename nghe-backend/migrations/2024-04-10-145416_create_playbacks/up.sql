-- Your SQL goes here
create table
playbacks (
    user_id uuid not null,
    song_id uuid not null,
    count integer not null default 1 check (count > 0),
    updated_at timestamptz not null default now(),
    constraint playbacks_pkey primary key (user_id, song_id),
    constraint playbacks_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade,
    constraint playbacks_song_id_fkey foreign key (
        song_id
    ) references songs (id) on delete cascade
);

select add_updated_at('playbacks');
