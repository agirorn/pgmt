use pgmt_core::test_migration;
use pgmt_core::tests_helper::get_table_names;
use pgmt_core::vec_of_string;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_migrating_from_files() {
    test_migration(vec!["tests/migrations"], None, async |pool| {
        assert_eq!(
            get_table_names(&pool).await,
            vec_of_string!["_schema_history", "table_1_name", "table_2_name",]
        );
    })
    .await;
}
