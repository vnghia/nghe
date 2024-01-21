-- Your SQL goes here
create table
  albums (
    id uuid not null default gen_random_uuid () constraint albums_pkey primary key,
    name text not null constraint albums_name_key unique,
    updated_at timestamptz not null default now()
  );

select
  diesel_manage_updated_at ('albums');