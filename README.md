# PostgreSQL Migration Tool (pgmt)

> PostgrSQL Database Migrations Made Easy.

## Install

```shell
cargo install pgmt
```

## Usage

To migrate a database using pgmt add add a singel migration to the migrations
folder

__edit:*./migrations/V1.0.0__Creat_table*__

```shell
CREATE TABLE new_table (
  name     TEXT      NOT NULL,
  "offset" BIGSERIAL NOT NULL
);
CREATE UNIQUE INDEX new_table_name_unique_index
    ON new_table_name(name,"offset");
``

Now to get the the new_table into the database, you should run this

```shell
# Migrate the data base
export PGMT_TEST_DB_URL=postgres://username:password@localhost:5432/database
pgmt migrate
```

## Help

```shell
pgmt --help
```
