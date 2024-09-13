-- Your SQL goes here
create function add_updated_at(_tbl regclass) returns void as $$
begin
    execute format('create trigger set_updated_at before update on %s
                    for each row execute procedure set_updated_at()', _tbl);
end;
$$ language plpgsql;

create function set_updated_at() returns trigger as $$
begin
    if (
        new is distinct from old and
        new.updated_at is not distinct from old.updated_at
    ) then
        new.updated_at := current_timestamp;
    end if;
    return new;
end;
$$ language plpgsql;

create function add_updated_at_leave_scanned_at(
    _tbl regclass
) returns void as $$
begin
    execute format('create trigger set_updated_at_leave_scanned_at before update on %s
                    for each row execute procedure set_updated_at_leave_scanned_at()', _tbl);
end;
$$ language plpgsql;

create function set_updated_at_leave_scanned_at() returns trigger as $$
begin
    if (
        new is distinct from old and
        new.updated_at is not distinct from old.updated_at and
        new.scanned_at is not distinct from old.scanned_at
    ) then
        new.updated_at := current_timestamp;
    end if;
    return new;
end;
$$ language plpgsql;
