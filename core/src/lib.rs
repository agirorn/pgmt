mod dao;
mod error;
mod template;
pub mod tests_helper;
use crate::dao::get_schema_history_rows;
pub use crate::error::{ChecksumMismatchError, Error, Result};
use chrono::Utc;
use crc32fast::Hasher as Crc32Hasher;
pub use deadpool_postgres::Pool;
use deadpool_postgres::{Client, Config, ManagerConfig, RecyclingMethod, Runtime};
use dotenvy::dotenv;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use template::fill_template;
use tokio_postgres::types::ToSql;
use tokio_postgres::NoTls; // Adjust module path
use url::Url;

pub type Placeholders = HashMap<String, String>;

/// Vector of values converted to String
#[macro_export]
macro_rules! vec_of_string {
    ($($x:expr),* $(,)?) => {
        vec![$($x.to_string()),*]
    };
}

macro_rules! to_sql_params {
    ($($x:expr),* $(,)?) => {
        &[$(
            &$x as &(dyn ToSql + Sync)
        ),*]
    };
}

use std::future::Future;

pub async fn migration_dirs<P>(
    migrations: Vec<P>,
    url: String,
    placeholders: Placeholders,
) -> Result<()>
where
    P: Into<String>,
{
    let migrations: Vec<String> = migrations.into_iter().map(Into::into).collect();
    let cfg = new_cfg(url);
    let files = read_sql_files(migrations.clone())?;
    println!("files: {files:#?}");
    let pool = create_pool(&cfg).await?;
    migrate(&pool, files, placeholders).await?;
    Ok(())
}

/// test_helper is a test helper that provisions a new database and migrates with the migrataion
/// paths provided and does a cleanup after the callback has compleated it's execution.
pub async fn test_migration<F, Fut, P, Output>(
    migrations: Vec<P>,
    placeholders: Option<Placeholders>,
    callback: F,
) -> Output
where
    P: Into<String>,
    F: FnOnce(Pool) -> Fut,
    Fut: Future<Output = Output>,
{
    dotenv().ok();
    let migrations: Vec<String> = migrations.into_iter().map(Into::into).collect();
    let db_url = std::env::var("PGMT_TEST_DB_URL")
        .expect("Environment variable PGMT_TEST_DB_URL is not set!");
    let db_name = generate_temp_db_name();
    let cfg = new_cfg(db_url.clone());
    let pool = setup(cfg.clone(), db_name.clone()).await.unwrap();
    let files = read_sql_files(migrations.clone()).unwrap();
    migrate(&pool, files, placeholders.unwrap_or_default())
        .await
        .unwrap();
    let result = callback(pool).await;
    teardown(db_url, &db_name).await.unwrap();
    result
}

/// test_db creata a new test dba and gives the user both a connecion to the database and the
/// conneciont url so they can connect to it from other tools
pub async fn test_db<F, Fut, Output>(callback: F) -> Output
where
    F: FnOnce(Pool, String) -> Fut,
    Fut: Future<Output = Output>,
{
    dotenv().ok();
    let db_url = std::env::var("PGMT_TEST_DB_URL")
        .expect("Environment variable PGMT_TEST_DB_URL is not set!");

    let mut url = Url::parse(&db_url).expect("Failed to parse URL");
    let db_name = generate_temp_db_name();
    url.set_path(&db_name);
    println!("###############################################");
    println!("URL: {}", url);
    println!("###############################################");
    let cfg = new_cfg(db_url.clone());
    let pool = setup(cfg.clone(), db_name.clone()).await.unwrap();
    let result = callback(pool, url.to_string()).await;
    teardown(db_url, &db_name).await.unwrap();
    result
}

/// migrate_files is a test helper that is most usefull just to test pgmt itself.
///
/// It provisions a new database and migrates with the migrataion
/// paths provided and does a cleanup after the callback has compleated it's execution.
pub async fn migrate_files<F, Fut, Output>(
    files: Vec<SqlFile>,
    placeholders: Option<Placeholders>,
    callback: F,
) -> Output
where
    F: FnOnce(Pool) -> Fut,
    Fut: Future<Output = Output>,
{
    dotenv().ok();
    let db_url = std::env::var("PGMT_TEST_DB_URL")
        .expect("Environment variable PGMT_TEST_DB_URL is not set!");
    let db_name = generate_temp_db_name();
    let cfg = new_cfg(db_url.clone());
    let pool = setup(cfg.clone(), db_name.clone()).await.unwrap();

    migrate(&pool, files, placeholders.unwrap_or_default())
        .await
        .unwrap();
    let result = callback(pool).await;
    teardown(db_url, &db_name).await.unwrap();
    result
}

async fn setup(mut cfg: Config, db_name: String) -> Result<Pool> {
    {
        let sql = format!("CREATE DATABASE {db_name}");
        get_client(&create_pool(&cfg).await?)
            .await?
            .execute(&sql, &[])
            .await?;
    }
    cfg.dbname = Some(db_name.clone());
    create_pool(&cfg).await
}

async fn create_pool(cfg: &Config) -> Result<Pool> {
    let pool: Pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|e| -> Error {
            format!("Unable to get a connection from the pool: {e}").into()
        })?;

    Ok(pool)
}

async fn get_client(pool: &Pool) -> Result<Client> {
    pool.get()
        .await
        .map_err(|e| -> Error { format!("Unable to get a connection from the pool: {e}").into() })
}

pub async fn migrate(
    pool: &Pool,
    files: Vec<SqlFile>,

    placeholders: HashMap<String, String>,
) -> Result<()> {
    let files: Vec<SqlInnerFile> = files.into_iter().map(SqlInnerFile::from).collect();
    let files = sort_sql_files(files);
    let files: Vec<SqlInnerFile> = files
        .into_iter()
        .filter(|f| matches!(f.kind(), Some(SqlFileKind::V(_))))
        .collect();
    println!("files X: {files:#?}");

    let client = get_client(pool).await?;
    create_schema_history_if_needed(&client).await?;
    let files = filter_out_and_verify_privious_migrations(&client, files).await?;

    println!("files: {files:#?}");
    for file in &files {
        // running each migration in order.
        // TODO: wrap this in a transaction
        // TODO: update the history table.
        // TODO: Make this available from the cli
        // TODO: Make this also do the beforeMigration hooks
        // TODO: Make this also do the Repeatable migrations if needed
        // TODO: expand placeholders from config
        // TODO: expand placeholders from enviroment variables
        // TODO: expand placeholders from config file ?? TOML
        // TODO: Allow rollback
        // TODO: Add front-matter into SQL file to allow disable transactions
        // TODO: Add front-matter into SQL file to allow disable transactions
        // TODO: Add partal update for only new migrations
        // TODO: Fail if the is a second pass and the checksum of the prefious pass failed
        // TODO: Fail if any of the migrations has been remove from the filesystem
        // TODO: Enshure the beforeMigration is run before any other migrataion and before the
        // checksum and exists checks

        client.query("BEGIN;", &[]).await?;

        let insert_into_schema_history_sql = r#"
           insert into _schema_history
                ( version
                , description
                , type
                , script
                , checksum
                , installed_by
                , installed_on
                , execution_time
                , success
                )
           VALUES
                (  $1 -- version
                ,  $2 -- description
                ,  $3 -- type
                ,  $4 -- script
                ,  $5 -- checksum
                ,  $6 -- installed_by
                ,  $7 -- installed_on
                ,  $8 -- execution_time
                ,  $9 -- success
              )
        "#;

        client
            .execute(
                insert_into_schema_history_sql,
                to_sql_params![
                    file.version,     // version
                    file.description, // description
                    file.prefix,      // type
                    file.file_name,   // script
                    file.checksum,    // checksum
                    "installed_by",   // installed_by
                    Utc::now(),       // installed_on
                    0,                // execution_time
                    true,             // success
                ],
            )
            .await?;

        let content = fill_template(&file.content, &placeholders)?;
        // We should handle error whew better and do a rollback if we can
        match client.batch_execute(&content).await {
            Ok(_) => {
                println!("OK");
                client.query("COMMIT;", &[]).await?;
                Ok(())
            }
            Err(e) => {
                println!("error {}", e);
                client.query("ROLLBACK;", &[]).await?;
                Err(e)
            }
        }?;
    }
    Ok(())
}

pub async fn teardown(db_url: String, db_name: &str) -> Result<()> {
    get_client(&create_pool(&new_cfg(db_url)).await?)
        .await?
        .execute(&format!("DROP DATABASE {} WITH (FORCE)", db_name), &[])
        .await?;
    Ok(())
}

fn new_cfg(db_url: String) -> Config {
    let mut cfg = Config::new();
    cfg.url = Some(db_url);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg
}

fn generate_temp_db_name() -> String {
    use chrono::Local;
    use rand::Rng;
    let now = Local::now();
    let timestamp = now.format("%Y_%m_%d_%H_%M_%S").to_string();

    let mut rng = rand::rng();
    let letters = b"abcdefghijklmnopqrstuvwxyz";
    let rand_string: String = (0..6)
        .map(|_| {
            let idx = rng.random_range(0..letters.len());
            letters[idx] as char
        })
        .collect();

    format!("pgmt_test_{}_{}", timestamp, rand_string)
}

#[derive(Debug, Clone)]
pub struct SqlFile {
    pub content: String,
    pub file_name: String,
    // TODO: should we delete the path here. I't probablay dose not matter.
    // It could be come usefull for error reporting later on.
    #[allow(dead_code)]
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct SqlInnerFile {
    pub content: String,
    pub checksum: i32,
    pub file_name: String,
    // TODO: should we delete the path here. I't probablay dose not matter.
    // It could be come usefull for error reporting later on.
    #[allow(dead_code)]
    pub file_path: String,
    pub prefix: String,
    pub version: Option<String>,
    pub description: String,
}

impl From<SqlFile> for SqlInnerFile {
    fn from(file: SqlFile) -> Self {
        let SqlFile {
            content,
            file_name,
            file_path,
        } = file;

        let version: Option<String>;
        let prefix: String;
        if let Some(rest) = file_name.strip_prefix('U') {
            version = Some(rest.split("__").next().unwrap().to_string());
            prefix = "U".to_string();
        } else if let Some(rest) = file_name.strip_prefix('V') {
            version = Some(rest.split("__").next().unwrap().to_string());
            prefix = "V".to_string();
        } else if let Some(rest) = file_name.strip_prefix('R') {
            version = Some(rest.split("__").next().unwrap().to_string());
            prefix = "U".to_string();
        } else {
            panic!("Unsuported prefix in {file_name}");
        };

        let mut hasher = Crc32Hasher::new();
        hasher.update(b"foo bar baz");
        let checksum = hasher.finalize() as i32;

        let description = file_name
            .clone()
            .split_once("__")
            .map(|(_, after)| after.replace('_', " "))
            .unwrap_or_default();

        // Normalize line endings (CRLF → LF)
        let content = content.replace("\r\n", "\n");

        Self {
            content,
            file_name,
            file_path,
            prefix,
            checksum,
            version,
            description,
        }
    }
}

// TODO: Add tests for this guy
fn read_sql_files<P>(dirs: Vec<P>) -> io::Result<Vec<SqlFile>>
where
    P: Into<String>,
{
    let mut files = Vec::new();
    for dir in dirs {
        let dir_str = dir.into();
        let path = Path::new(&dir_str);

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let file_path: PathBuf = entry.path();
                let file_name: String = entry.file_name().to_str().unwrap().to_string();

                if file_path.is_file() {
                    if let Some(ext) = file_path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "sql" {
                            let content = fs::read_to_string(&file_path)?;
                            files.push(SqlFile {
                                content,
                                file_name,
                                file_path: file_path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}

use semver::Version;
use std::cmp::Ordering;

// Extract a sort key enum
#[derive(Debug)]
enum SqlFileKind {
    U(Version),
    V(Version),
    R(String),
}

impl SqlInnerFile {
    // TODO: Add stom tests for this
    fn kind(&self) -> Option<SqlFileKind> {
        let name = self.file_name.as_str();

        if let Some(rest) = name.strip_prefix('U') {
            let version_str = rest.split("__").next()?;
            Version::parse(version_str).ok().map(SqlFileKind::U)
        } else if let Some(rest) = name.strip_prefix('V') {
            let version_str = rest.split("__").next()?;
            Version::parse(version_str).ok().map(SqlFileKind::V)
        } else if name.starts_with('R') {
            Some(SqlFileKind::R(name.to_string()))
        } else {
            None // not sortable
        }
    }
}

fn sort_sql_files(mut files: Vec<SqlInnerFile>) -> Vec<SqlInnerFile> {
    files.sort_by(|a, b| {
        match (a.kind(), b.kind()) {
            (Some(SqlFileKind::U(a)), Some(SqlFileKind::U(b))) => a.cmp(&b),
            (Some(SqlFileKind::V(a)), Some(SqlFileKind::V(b))) => a.cmp(&b),
            (Some(SqlFileKind::R(a)), Some(SqlFileKind::R(b))) => a.cmp(&b),
            // Ordering between kinds: U < V < R
            (Some(SqlFileKind::U(_)), _) => Ordering::Less,
            (_, Some(SqlFileKind::U(_))) => Ordering::Greater,
            (Some(SqlFileKind::V(_)), _) => Ordering::Less,
            (_, Some(SqlFileKind::V(_))) => Ordering::Greater,
            _ => Ordering::Equal, // If one or both couldn't be classified
        }
    });

    files
}

async fn create_schema_history_if_needed(client: &Client) -> Result<()> {
    let sql = r#"
        SELECT exists (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public'  -- or another schema name
              AND table_name = '_schema_history'
        ) AS exists;
    "#;
    let exists: bool = client.query_one(sql, &[]).await?.try_get("exists")?;
    if !exists {
        create_schema_history_table(client).await?;
    }
    Ok(())
}

async fn create_schema_history_table(client: &Client) -> Result<()> {
    client
        .batch_execute(
            r#"
              CREATE TABLE _schema_history (
                    -- Auto-incrementing rank (used as primary key and order of migration)
                    installed_rank SERIAL PRIMARY KEY,

                    -- Version of the migration (e.g., 1.2, 2.0)
                    version VARCHAR(50),

                    -- Human-readable description (e.g., Create users table)
                    description VARCHAR(200) NOT NULL,

                    -- Type of migration (SQL, JDBC, UNDO, etc.)
                    type VARCHAR(20) NOT NULL,

                    -- Filename of the migration script
                    script VARCHAR(1000) NOT NULL,

                    -- Checksum used to detect script changes
                    checksum INT,

                    -- Database user who ran the migration
                    installed_by VARCHAR(100) NOT NULL,

                    -- Timestamp when the migration was applied
                    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),

                    -- Time in milliseconds to execute the migration
                    execution_time INT NOT NULL,

                    -- Whether the migration succeeded (true) or failed (false)
                    success BOOLEAN NOT NULL
              );


              COMMENT ON COLUMN _schema_history .installed_rank
                   IS 'Execution order rank (primary key); increments with each migration';

              COMMENT ON COLUMN _schema_history .version
                   IS 'Version of the migration (e.g., 1.0, 2.1.3). Null for repeatable migrations';

              COMMENT ON COLUMN _schema_history .description
                   IS 'Human-readable description of the migration (e.g., Create users table)';

              COMMENT ON COLUMN _schema_history .type
                   IS 'Type of migration (e.g., SQL, JDBC, REPEATABLE, UNDO)';

              COMMENT ON COLUMN _schema_history .script
                   IS 'Name of the migration script file';

              COMMENT ON COLUMN _schema_history .checksum
                   IS 'Checksum of the migration script content to detect changes. Null for repeatable if not validated';

              COMMENT ON COLUMN _schema_history .installed_by
                   IS 'Database user who applied the migration';

              COMMENT ON COLUMN _schema_history .installed_on
                   IS 'Timestamp when the migration was applied. Defaults to current time';

              COMMENT ON COLUMN _schema_history .execution_time
                   IS 'Execution time of the migration in milliseconds';

              COMMENT ON COLUMN _schema_history .success
                   IS 'Whether the migration was successful (true) or failed (false)';


            "#,
        )
        .await?;

    Ok(())
}

async fn filter_out_and_verify_privious_migrations(
    client: &Client,
    files: Vec<SqlInnerFile>,
) -> Result<Vec<SqlInnerFile>> {
    let mut schema_history = get_schema_history_rows(client).await;
    let mut result: Vec<SqlInnerFile> = vec![];

    for file in files {
        if schema_history.is_empty() {
            result.push(file);
        } else {
            let history = schema_history.remove(0);
            if file.checksum != history.checksum {
                return Err(Error::ChecksumMismatchError(ChecksumMismatchError {
                    file_name: file.file_name,
                    file_checksum: file.checksum,
                    applied_checksum: history.checksum,
                }));
            }
        }
    }
    Ok(result)
}
