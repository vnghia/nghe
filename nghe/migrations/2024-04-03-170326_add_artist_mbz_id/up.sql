-- Your SQL goes here
alter table artists add mbz_id uuid,
add constraint artists_mbz_id_key unique (mbz_id);

create unique index artists_name_idx on artists (name) where (mbz_id is null);

alter table artists drop constraint artists_name_key;
