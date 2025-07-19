use crate::Pool;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::convert::TryFrom;
use tokio_postgres::Row;

pub async fn get_table_names(pool: &Pool) -> Vec<String> {
    let client = pool
        .get()
        .await
        .expect("Unable to get a connection from the pool");
    let sql = r#"
        SELECT table_name
          FROM information_schema.tables
         WHERE table_schema='public';
    "#;
    if let Ok(rows) = client.query(sql, &[]).await {
        return rows
            .into_iter()
            .map(|row| row.get("table_name"))
            .collect::<Vec<String>>();
    }
    vec![]
}

pub async fn get_table_columns(pool: &Pool, table_name: &String) -> Vec<TableColumn> {
    let client = pool
        .get()
        .await
        .expect("Unable to get a connection from the pool");

    let sql = r#"
        SELECT cols.column_name
             , cols.data_type
             , cols.is_nullable
             , cols.column_default
             , pgd.description AS comment
          FROM information_schema.columns cols
          JOIN pg_catalog.pg_class c ON c.relname = cols.table_name
          JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
          LEFT JOIN pg_catalog.pg_description pgd
            ON pgd.objoid = c.oid
           AND pgd.objsubid = cols.ordinal_position
         WHERE cols.table_name = $1
           AND cols.table_schema = 'public'
           AND n.nspname = 'public'
         ORDER BY cols.column_name;
    "#;
    if let Ok(rows) = client.query(sql, &[table_name]).await {
        let rows: Vec<TableColumn> = rows
            .into_iter()
            .map(TableColumn::try_from)
            .collect::<Result<_, _>>()
            .unwrap();
        return rows;
    }
    vec![]
}

#[derive(Debug, Serialize, PartialEq)]
pub struct TableColumn {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub comment: Option<String>,
}

impl TryFrom<Row> for TableColumn {
    type Error = tokio_postgres::Error;

    /// Try to convert from Row into TableColumn
    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(TableColumn {
            column_name: row.try_get("column_name")?,
            data_type: row.try_get("data_type")?,
            is_nullable: row.try_get::<&str, String>("is_nullable")? == "YES",
            column_default: row.try_get("column_default")?,
            comment: row.try_get("comment")?,
        })
    }
}

pub fn new_schema_history_columns() -> Vec<TableColumn> {
    vec![

        TableColumn {
            column_name: "checksum".to_string(),
            data_type: "integer".to_string(),
            is_nullable: true,
            column_default: None,
            comment: Some(
                "Checksum of the migration script content to detect changes. Null for repeatable if not validated".to_string(),
            ),
        },
        TableColumn {
            column_name: "description".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: false,
            column_default: None,
            comment: Some(
                "Human-readable description of the migration (e.g., Create users table)".to_string(),
            ),
        },
        TableColumn {
            column_name: "execution_time".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            column_default: None,
            comment: Some(
                "Execution time of the migration in milliseconds".to_string(),
            ),
        },
        TableColumn {
            column_name: "installed_by".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: false,
            column_default: None,
            comment: Some(
                "Database user who applied the migration".to_string(),
            ),
        },
        TableColumn {
            column_name: "installed_on".to_string(),
            data_type: "timestamp with time zone".to_string(),
            is_nullable: false,
            column_default: Some(
                "now()".to_string(),
            ),
            comment: Some(
                "Timestamp when the migration was applied. Defaults to current time".to_string(),
            ),
        },
        TableColumn {
            column_name: "installed_rank".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            column_default: Some(
                "nextval('_schema_history_installed_rank_seq'::regclass)".to_string(),
            ),
            comment: Some(
                "Execution order rank (primary key); increments with each migration".to_string(),
            ),
        },
        TableColumn {
            column_name: "script".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: false,
            column_default: None,
            comment: Some(
                "Name of the migration script file".to_string(),
            ),
        },
        TableColumn {
            column_name: "success".to_string(),
            data_type: "boolean".to_string(),
            is_nullable: false,
            column_default: None,
            comment: Some(
                "Whether the migration was successful (true) or failed (false)".to_string(),
            ),
        },
        TableColumn {
            column_name: "type".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: false,
            column_default: None,
            comment: Some(
                "Type of migration (e.g., SQL, JDBC, REPEATABLE, UNDO)".to_string(),
            ),
        },
        TableColumn {
            column_name: "version".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: true,
            column_default: None,
            comment: Some(
                "Version of the migration (e.g., 1.0, 2.1.3). Null for repeatable migrations".to_string(),
            ),
        },
    ]
}

pub async fn get_schema_history_rows(pool: &Pool) -> Vec<SchemaHistoryRow> {
    let client = pool
        .get()
        .await
        .expect("Unable to get a connection from the pool");

    let sql = r#"
        SELECT installed_rank
             , version
             , description
             , type
             , script
             , checksum
             , installed_by
             , installed_on
             , execution_time
             , success
          FROM _schema_history
         ORDER BY installed_rank;
    "#;
    if let Ok(rows) = client.query(sql, &[]).await {
        let rows: Vec<SchemaHistoryRow> = rows
            .into_iter()
            .map(SchemaHistoryRow::try_from)
            .collect::<Result<_, _>>()
            .unwrap();
        return rows;
    }
    vec![]
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SchemaHistoryRow {
    /// Auto-incrementing rank (used as primary key and order of migration)
    pub installed_rank: i32,

    /// Version of the migration (e.g., 1.2, 2.0)
    pub version: Option<String>,

    /// Human-readable description (e.g., Create users table)
    pub description: String,

    /// Type of migration (SQL, JDBC, UNDO, etc.)
    pub r#type: String,

    /// Filename of the migration script
    pub script: String,

    /// Checksum used to detect script changes
    pub checksum: i32,

    /// Database user who ran the migration
    pub installed_by: String,

    /// Timestamp when the migration was applied
    pub installed_on: DateTime<Utc>,

    /// Time in milliseconds to execute the migration
    pub execution_time: i32,

    /// Whether the migration succeeded (true) or failed (false)
    pub success: bool,
}

impl TryFrom<Row> for SchemaHistoryRow {
    type Error = tokio_postgres::Error;

    /// Try to convert from Row into TableColumn
    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(SchemaHistoryRow {
            installed_rank: row.try_get("installed_rank")?,
            version: row.try_get("version")?,
            description: row.try_get("description")?,
            r#type: row.try_get("type")?,
            script: row.try_get("script")?,
            checksum: row.try_get("checksum")?,
            installed_by: row.try_get("installed_by")?,
            installed_on: row.try_get("installed_on")?,
            execution_time: row.try_get("execution_time")?,
            success: row.try_get("success")?,
        })
    }
}
