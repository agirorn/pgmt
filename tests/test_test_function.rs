use pgmt::{test_migration, tests_helper::get_table_names, vec_of_string};

#[tokio::test]
async fn test_fuction() {
    test_migration(vec!["core/tests/migrations"], None, async |pool| {
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1_name", "table_2_name",]
        );
    })
    .await;
}
