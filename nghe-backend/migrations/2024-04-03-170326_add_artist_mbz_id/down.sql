-- This file should undo anything in `up.sql`
alter table artists
add constraint artists_name_key unique (name);

drop index artists_name_idx;

alter table artists drop column mbz_id;
