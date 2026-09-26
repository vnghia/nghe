-- Your SQL goes here
create table playqueues (
    user_id uuid not null constraint playqueues_pkey primary key,
    ids uuid [] not null check (array_position(ids, null) is null),
    current uuid,
    position bigint,
    constraint playqueues_user_id_fkey foreign key (
        user_id
    ) references users (id) on delete cascade
);
