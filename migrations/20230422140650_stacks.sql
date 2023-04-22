-- Add migration script here

create table if not exists stack_tags (
	tag text primary key
);

insert into stack_tags(tag) values ("GLOBAL");

create table if not exists stacked_notes (
	stack_tag text not null,
    stack_index integer not null,
    note text not null,

    PRIMARY KEY (stack_tag, stack_index),
    unique (stack_tag, note),

    FOREIGN KEY(stack_tag) REFERENCES stack_tags(tag) on delete cascade on update cascade,
    FOREIGN KEY(note) REFERENCES notes(name) on delete cascade on update cascade );
