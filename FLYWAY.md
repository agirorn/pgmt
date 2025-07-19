# Flyay

__Why are we supporting flyway features?__

To assit Flyway users transition from flyway over to PGMT.

__Why not simply use Flyway?__

flyway has a nomber of drabacks that make it less than optimal for large and
groinw code bases like:
- It's deployment is large cointing hunderds of begabites in when deployed in
  docker.
- It's slow to start compated to native code.
- It's users a lot of memmory
- It does not ahve any build in tooling for the lifecile of databas migrations.
- It has a clumsy cli UX using the default java argument processing.
- Has not gotten any nice new featurs in a long time.

PGMT is not for the users of Flyway that are 

### Flyway Naming

We are supporting the flyway naming but will not support any of ther
shenanigans like not throint error when files not fitting wihte the pattersn are
planced in the locations. See https://www.red-gate.com/blog/database-devops/flyway-naming-patterns-matter

We will support files that start with

- V<version>__name.sql -> Normal migration
- U<version>__name.sql -> Undo migrations
- R<version>__name.sql -> Repeatable migrations (Whata a lie)

### Repeatable migrations (Whata a lie)
They are not truly repeatable since they are only run when they chagnes and thy
must be triked wihte some date time shanananges to run all the time.

From some perspective, they could be considers repetable as they run every time
the changes but most people I have met did not understant this as Flyway defines
this.

### Hooks

_We will also support the hooks atleast the beforeMigrate_
Excample: beforeMigrate.sql



# Flyway schema history table
https://documentation.red-gate.com/fd/flyway-schema-history-table-273973417.html

We will store things in the flyway history table if we need to.
We will support convert from to PGMT history table
