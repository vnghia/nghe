-- This file should undo anything in `up.sql`
drop function add_updated_at(_tbl regclass);

drop function set_updated_at();

drop function add_updated_at_leave_scanned_at(_tbl regclass);

drop function set_updated_at_leave_scanned_at();
