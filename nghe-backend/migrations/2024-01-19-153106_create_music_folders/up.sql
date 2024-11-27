-- Your SQL goes here
create table
music_folders (
    id uuid not null default gen_random_uuid() constraint music_folders_pkey primary key,
    path text not null constraint music_folders_path_key unique,
    scanned_at timestamptz not null default now()
);
