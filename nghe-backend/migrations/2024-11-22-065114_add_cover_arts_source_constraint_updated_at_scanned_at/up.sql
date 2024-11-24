-- Your SQL goes here
alter table cover_arts
add column source text,
add column updated_at timestamptz not null default now(),
add column scanned_at timestamptz not null default now(),
drop column upserted_at,
add constraint cover_arts_file_hash_file_size_key unique (
    file_hash, file_size
),
drop constraint cover_arts_format_file_hash_file_size_key;
