-- Your SQL goes here
alter table cover_arts
add column source text,
add column updated_at timestamptz not null default now(),
add column scanned_at timestamptz not null default now(),
drop column upserted_at,
add constraint cover_arts_source_file_hash_file_size_key
unique nulls not distinct (
    source, file_hash, file_size
),
drop constraint cover_arts_format_file_hash_file_size_key;

select add_updated_at_leave_scanned_at('cover_arts');
