-- This file should undo anything in `up.sql`
drop trigger set_updated_at_leave_scanned_at on music_folders;

alter table music_folders drop column updated_at;

alter table music_folders drop column name;
