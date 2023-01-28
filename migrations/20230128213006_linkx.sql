-- Add migration script here

CREATE TABLE linkx (
  _from text not null, 
  _to text not null, 
  PRIMARY KEY (_from, _to),

    FOREIGN KEY(_from) REFERENCES notes(name) on delete cascade on update cascade,
    FOREIGN KEY(_to) REFERENCES notes(name) on delete cascade on update cascade );
CREATE INDEX _to_index ON linkx(_to);