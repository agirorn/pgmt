use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use serde::Serialize;
use std::convert::TryFrom;
use tokio_postgres::Row;

pub async fn get_schema_history_rows(client: &Client) -> Vec<SchemaHistoryRow> {
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
