-- Your SQL goes here
create table
  artists (
    id uuid not null default gen_random_uuid () constraint artists_pkey primary key,
    name text not null constraint artists_name_key unique,
    index text not null default '?',
    updated_at timestamptz not null default now()
  );

select
  add_updated_at ('artists');