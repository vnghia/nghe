-- This file should undo anything in `up.sql`
alter table artists alter column index set default '?';
