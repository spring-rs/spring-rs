-- Your SQL goes here

CREATE TABLE IF NOT EXISTS public.users
(
    id bigint NOT NULL,
    name character varying(20) COLLATE pg_catalog."default",
    active boolean DEFAULT false,
    CONSTRAINT users_pkey PRIMARY KEY (id)
);


insert into public.users (id, name, active) values (1, 'Alice', true);
insert into public.users (id, name, active) values (2, 'Bob', false);
insert into public.users (id, name, active) values (3, 'Charlie', true);
