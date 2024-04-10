-- Your SQL goes here
create table
playbacks (
    user_id uuid not null,
    song_id uuid not null,
    count integer not null default 1 check (count > 0),
    updated_at timestamptz not null default now(),
    constraint playbacks_pkey primary key (user_id, song_id)
);

select add_updated_at('playbacks');
