-- Your SQL goes here
create table
scans (
    started_at timestamptz not null default now() constraint scans_pkey primary key,
    is_scanning boolean not null default true,
    finished_at timestamptz default null,
    scanned_count bigint not null default 0,
    error_message text default null,
    check (
        (
            is_scanning
            and finished_at is null
        )
        or (
            not is_scanning
            and finished_at is not null
        )
    )
);

create unique index scans_is_scanning_idx on scans (is_scanning)
where
is_scanning;
