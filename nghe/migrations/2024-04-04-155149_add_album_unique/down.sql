-- This file should undo anything in `up.sql`
alter table albums
add constraint albums_name_key unique (name);

drop index albums_name_date_release_original_idx;

alter table albums drop column mbz_id;

alter table albums drop column year;
alter table albums drop column month;
alter table albums drop column day;
alter table albums drop column release_year;
alter table albums drop column release_month;
alter table albums drop column release_day;
alter table albums drop column original_release_year;
alter table albums drop column original_release_month;
alter table albums drop column original_release_day;
