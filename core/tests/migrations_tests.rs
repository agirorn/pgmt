use chrono::{DateTime, Utc};
use pgmt_core::tests_helper::{
    get_schema_history_rows, get_table_columns, get_table_names, new_schema_history_columns,
    SchemaHistoryRow, TableColumn,
};
use pgmt_core::{migrate, migrate_files, vec_of_string, Placeholders, SqlFile};
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_migrate_files() {
    let files = vec![
        SqlFile {
            content: "DROP TABLE klines;".into(),
            file_name: "U1.0.0__Drop_table_1_name.sql".into(),
            file_path: "migrations/U1.0.0__Drop_table_1_name.sql".into(),
        },
        SqlFile {
            content: r#"
                CREATE TABLE table_1_name (
                  name     TEXT      NOT NULL,
                  "offset" BIGSERIAL NOT NULL
                );
                CREATE UNIQUE INDEX table_1_name_unique_index
                    ON table_1_name(name,"offset");
            "#
            .into(),
            file_name: "V1.0.0__Create_table_1_name.sql".into(),
            file_path: "migrations/V1.0.0__Create_table_1_name.sql".into(),
        },
        SqlFile {
            content: r#"
                CREATE TABLE table_2_name (
                  name     TEXT      NOT NULL,
                  closed   BOOL      NOT NULL,
                  "offset" BIGSERIAL NOT NULL
                );
            "#
            .into(),
            file_name: "V1.0.1__Add_table_2_name.sql".into(),
            file_path: "migrations/V1.0.1__Add_table_2_name.sql".into(),
        },
        SqlFile {
            content: "DROP TABLE table_2_name;".into(),
            file_name: "U1.0.1__Drop_table_2_name.sql".into(),
            file_path: "migrations/U1.0.1__Drop_table_2_name.sql".into(),
        },
    ];

    migrate_files(files, None, async |pool| {
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1_name", "table_2_name",]
        );

        assert_eq!(
            get_table_columns(&pool, &"_schema_history".to_string()).await,
            new_schema_history_columns(),
        );
    })
    .await;
}

#[tokio::test]
async fn failed_migration_does_not_change_the_database() {
    let files = vec![SqlFile {
        content: r#"
                CREATE TABLE users (id INT);
                CREATE TABLE users (id INT); -- fails
            "#
        .into(),
        file_name: "V1.0.0__Create_table_1_name.sql".into(),
        file_path: "migrations/V1.0.0__Create_table_1_name.sql".into(),
    }];

    migrate_files(vec![], None, async |pool| {
        let res = migrate(&pool, files, Placeholders::new()).await;
        assert!(res.is_err());

        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history"]
        );

        // The _schema_history is created before the first migration and this will be there
        assert_eq!(
            get_table_columns(&pool, &"_schema_history".to_string()).await,
            new_schema_history_columns(),
        );
    })
    .await;
}

#[tokio::test]
async fn only_migrate_the_latest_file() {
    let file_1 = SqlFile {
        content: r#"
                CREATE TABLE table_1 (id INT);
            "#
        .into(),
        file_name: "V1.0.0__migration.sql".into(),
        file_path: "migrations/V1.0.0__migration.sql".into(),
    };
    let files_1 = vec![file_1];

    migrate_files(vec![], None, async |pool| {
        let res = migrate(&pool, files_1, Placeholders::new()).await;
        assert!(res.is_ok());
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1"]
        );

        let date_time: DateTime<Utc> = "2025-05-17T21:02:39.341010Z".parse().unwrap();
        let rows = get_schema_history_rows(&pool).await;
        let rows: Vec<SchemaHistoryRow> = rows
            .into_iter()
            .map(|mut f| {
                // Overwrite installed_on sicne we are not testint he time stamp
                f.installed_on = date_time;
                f
            })
            .collect();
        assert_eq!(
            rows,
            vec![SchemaHistoryRow {
                installed_rank: 1,
                version: Some("1.0.0".to_string()),
                description: "migration.sql".to_string(),
                r#type: "V".to_string(),
                script: "V1.0.0__migration.sql".to_string(),
                checksum: -228401567,
                installed_by: "installed_by".to_string(),
                installed_on: date_time,
                execution_time: 0,
                success: true,
            }]
        );
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1"]
        );

        // The _schema_history is created before the first migration and this will be there
        assert_eq!(
            get_table_columns(&pool, &"_schema_history".to_string()).await,
            new_schema_history_columns(),
        );
    })
    .await;
}

#[tokio::test]
async fn some_test_find_me_a_better_name() {
    let migration_1 = SqlFile {
        content: r#"
                CREATE TABLE table_1 (id INT);
            "#
        .into(),
        file_name: "V1.0.0__migration.sql".into(),
        file_path: "migrations/V1.0.0__migration.sql".into(),
    };
    let migration_2 = SqlFile {
        content: r#"
            -- This migration enshures they are run in order since it
            -- will fail if it is run before table_1 has been created
            ALTER TABLE table_1
            ADD COLUMN name TEXT NOT NULL;
            "#
        .into(),
        file_name: "V1.1.0__migration.sql".into(),
        file_path: "migrations/V1.1.0__migration.sql".into(),
    };
    let files_1 = vec![migration_1.clone()];
    let files_2 = vec![
        // Setting migration 2 before migration 1 to verify that
        // the migrataion are run in version order and not the order they are
        // fead into the migration since they can come in any order form the filesystem
        migration_2,
        migration_1,
    ];

    migrate_files(vec![], None, async |pool| {
        let res = migrate(&pool, files_1, Placeholders::new()).await;
        assert!(res.is_ok());
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1"]
        );
        let res = migrate(&pool, files_2, Placeholders::new()).await;
        assert!(res.is_ok());
        assert_eq!(
            get_table_names(&pool).await,
            // vec!["_schema_history".to_string(), "table_1".to_string(),]
            vec_of_string!["_schema_history", "table_1"]
        );
        assert_eq!(
            get_table_columns(&pool, &"table_1".to_string()).await,
            vec![
                TableColumn {
                    column_name: "id".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: true,
                    column_default: None,
                    comment: None,
                },
                TableColumn {
                    column_name: "name".to_string(),
                    data_type: "text".to_string(),
                    is_nullable: false,
                    column_default: None,
                    comment: None,
                },
            ]
        );
    })
    .await;
}

// TODOs
// - test both files with CRLF and LF to make shure the content is normalized.
