-- This file should undo anything in `up.sql`
alter table cover_arts
drop column source,
drop column updated_at,
drop column scanned_at,
add column upserted_at timestamptz not null default now(),
add constraint cover_arts_format_file_hash_file_size_key unique (
    format, file_hash, file_size
);
