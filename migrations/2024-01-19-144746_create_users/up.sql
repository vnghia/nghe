-- Your SQL goes here
create table
  users (
    id uuid not null default gen_random_uuid () constraint users_pkey primary key,
    username text not null constraint users_username_key unique,
    password bytea not null,
    email text not null,
    admin_role boolean not null default false,
    download_role boolean not null default false,
    share_role boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
  );

select
  diesel_manage_updated_at ('users');