# TODO for pgmt

 1. Fail the migration if there are any files in the migration folder that cant be
   used. This is a much better user experience than failing silently.

 2. Add down migration support

 3. Add more tests to the CLI, like error checking for bad files, failed
   migrations and such. No reason not to test for these on every release.

 4. Improve the UX on a successful migration. Like printing the migrated file
   name to the console. And if there is noting to migrate print that.

 5. Add support to check what migration are pending.

 6. Fail the migration if an out of order migration is added.

 6. Add support for migration multiple schemas in the same database from
    multiple independent migration folders.

10. Add support for defining the migration path in an environment variable,
    something like PGMT_TEST_MIGRATION_PATH. This can then be used in modules
    that need point to migration out side of the default migration path. This
    environment variable should support a single path, of JSON object for path
    and optional place holders and a JSON array of objects for multiple paths
    for multiple schema migrations. It would also probably be nice to also
    support a file path to file that has this same capabilities and also to have
    a default file that would be piked up automatically.
