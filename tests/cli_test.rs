use assert_cmd::Command;
use indoc::indoc;
use pgmt_core::tests_helper::get_table_names;
use pgmt_core::vec_of_string;

#[test]
fn help() {
    Command::cargo_bin("pgmt")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(indoc! {"
            PostgreSQL Migration Tool

            Usage: pgmt <COMMAND>

            Commands:
              migrate  Run database migrations from one or more directories
              help     Print this message or the help of the given subcommand(s)

            Options:
              -h, --help  Print help
            "
        });
}

#[test]
fn migration_help() {
    Command::cargo_bin("pgmt")
        .unwrap()
        .args(vec!["migrate", "--help"])
        .assert()
        .success()
        .stdout(indoc! {"
            Run database migrations from one or more directories

            Usage: pgmt migrate --url <URL> <DIRECTORIES>...

            Arguments:
              <DIRECTORIES>...  Directories containing migrations

            Options:
              -u, --url <URL>  Database URL
              -h, --help       Print help
            "
        });
}

#[tokio::test]
async fn cli_migration() {
    pgmt_core::test_db(async |pool, url| {
        Command::cargo_bin("pgmt")
            .unwrap()
            .args(vec!["migrate", "--url", &url, "core/tests/migrations"])
            .assert()
            .success();
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1_name", "table_2_name",]
        );
    })
    .await;
}
