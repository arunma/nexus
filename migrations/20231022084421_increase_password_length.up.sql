-- Add up migration script here
alter table users alter column password TYPE VARCHAR(300);