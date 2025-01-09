-- Your SQL goes here
create table user_sessions (
    id uuid not null default gen_random_uuid() constraint user_sessions_pkey primary key,
    user_id uuid not null,
    created_at timestamptz not null default now(),
    constraint user_sessions_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade
);

create index user_sessions_user_id_idx on user_sessions (user_id);
