-- This file should undo anything in `up.sql`
alter table artists drop column ts;

alter table albums drop column ts;

alter table songs drop column ts;

drop text search configuration usimple;

drop extension unaccent;
