
CREATE TABLE IF NOT EXISTS users
(
    id bigint NOT NULL,
    name character varying(20) NOT NULL,
    active boolean DEFAULT false,
    CONSTRAINT users_pkey PRIMARY KEY (id)
);


insert into users (id, name, active) values (1, 'Alice', true);
insert into users (id, name, active) values (2, 'Bob', false);
insert into users (id, name, active) values (3, 'Charlie', true);
