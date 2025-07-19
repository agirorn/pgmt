CREATE TABLE table_1_name (
  name     TEXT      NOT NULL,
  "offset" BIGSERIAL NOT NULL
);
CREATE UNIQUE INDEX table_1_name_unique_index
    ON table_1_name(name,"offset");
