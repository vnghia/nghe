-- Your SQL goes here
create table
  configs (
    key text not null constraint configs_pkey primary key,
    text text default null,
    byte bytea default null,
    updated_at timestamptz not null default now()
  );

select
  add_updated_at ('configs');