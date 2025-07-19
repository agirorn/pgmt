use pgmt::{tests_helper::get_table_names, vec_of_string};

#[pgmt::test]
async fn test_no_args(pool: pgmt::Pool) {
    assert_eq!(
        get_table_names(&pool).await,
        vec_of_string!["_schema_history"]
    );
}

#[pgmt::test(migrations = "core/tests/migrations")]
async fn test_only_migrations_args(pool: pgmt::Pool) {
    assert_eq!(
        get_table_names(&pool).await,
        vec_of_string!["_schema_history", "table_1_name", "table_2_name"]
    );
}

#[pgmt::test(placeholder_env = "env_value")]
async fn test_fuction(pool: pgmt::Pool) {
    assert_eq!(
        get_table_names(&pool).await,
        vec_of_string!["_schema_history"]
    );
}

#[pgmt::test(
    migrations = "core/tests/migrations",
    placeholder_env = "env_value",
    placeholder_env_2 = "env_2_value"
)]
async fn test_fuction_2(pool: pgmt::Pool) {
    assert_eq!(
        get_table_names(&pool).await,
        vec_of_string!["_schema_history", "table_1_name", "table_2_name",]
    );
}
