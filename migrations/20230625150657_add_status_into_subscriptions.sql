-- Add migration script here
alter table subscriptions add column status text not null;
