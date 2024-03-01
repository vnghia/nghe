-- Your SQL goes here
create table
  albums (
    id uuid not null default gen_random_uuid () constraint albums_pkey primary key,
    name text not null constraint albums_name_key unique,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    scanned_at timestamptz not null default now()
  );

select
  add_updated_at_leave_scanned_at ('albums');