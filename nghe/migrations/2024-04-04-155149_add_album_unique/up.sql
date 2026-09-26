-- Your SQL goes here
alter table albums add year smallint default null;
alter table albums add month smallint default null;
alter table albums add day smallint default null;
alter table albums add release_year smallint default null;
alter table albums add release_month smallint default null;
alter table albums add release_day smallint default null;
alter table albums add original_release_year smallint default null;
alter table albums add original_release_month smallint default null;
alter table albums add original_release_day smallint default null;

alter table albums add mbz_id uuid,
add constraint albums_mbz_id_key unique (mbz_id);

create unique index albums_name_date_release_original_idx on albums (
    name,
    year,
    month,
    day,
    release_year,
    release_month,
    release_day,
    original_release_year,
    original_release_month,
    original_release_day
) nulls not distinct where (mbz_id is null);

alter table albums drop constraint albums_name_key;
