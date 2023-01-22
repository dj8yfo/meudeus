-- Add migration script here
-- Add migration script here
create table if not exists notes (
	name text primary key,
	filename text,

	unique(filename)
);
