mod common;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
pub mod tests {
    use hare_data_plane_client::error::DataPlaneResult;

    #[cfg(feature = "postgres")]
    use hare_data_plane_client::postgres::HareDataPlanePostgres;
    #[cfg(feature = "sqlite")]
    use hare_data_plane_client::sqlite::HareDataPlaneSqlite;

    use crate::common;

    #[sqlx::test]
    async fn test_when_bootstrap_then_ok() -> DataPlaneResult<()> {
        let database_url = std::env::var("DATABASE_URL").expect("Must provide DATABASE_URL environment variable");

        #[cfg(feature = "postgres")]
        let client = HareDataPlanePostgres::try_from_url(database_url).await?;
        #[cfg(feature = "sqlite")]
        let client = HareDataPlaneSqlite::try_from_url(database_url).await?;

        common::test_when_bootstrap_then_ok(client).await
    }

    #[cfg(feature = "postgres")]
    macro_rules! define_test {
        ($test_name:ident) => {
            #[sqlx::test]
            async fn $test_name(connection: sqlx::Pool<sqlx::Postgres>) -> DataPlaneResult<()> {
                let client = HareDataPlanePostgres::from_connection(connection);
                common::$test_name(&client).await
            }
        };
    }

    #[cfg(feature = "sqlite")]
    macro_rules! define_test {
        ($test_name:ident) => {
            #[sqlx::test]
            async fn $test_name(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
                let client = HareDataPlaneSqlite::from_connection(connection);
                common::$test_name(&client).await
            }
        };
    }

    define_test!(test_add_aliases_for_shortcut_when_add_single_alias_then_exists);
    define_test!(test_add_aliases_for_shortcut_when_add_multiple_distinct_aliases_then_exists);
    define_test!(test_add_aliases_for_shortcut_when_add_multiple_aliases_with_dupes_then_fail);
    define_test!(test_add_aliases_for_shortcut_when_add_to_non_existent_shortcut_then_fail);
    define_test!(test_add_aliases_for_shortcut_when_add_existing_alias_then_fail);
    define_test!(test_add_aliases_for_shortcut_when_add_multiple_one_existing_alias_then_fail);
    define_test!(test_create_shortcut_when_invalid_url_then_fail);
    define_test!(test_create_shortcut_when_valid_url_then_exists);
    define_test!(test_create_shortcut_when_url_already_exist_then_fail);
    define_test!(test_create_shortcut_when_empty_aliases_then_fail);
    define_test!(test_delete_aliases_when_empty_aliases_then_fail);
    define_test!(test_delete_aliases_when_non_existent_shortcut_then_fail);
    define_test!(test_delete_aliases_when_single_alias_one_existing_then_fail);
    define_test!(test_delete_aliases_when_more_than_existing_then_fail);
    define_test!(test_delete_aliases_when_one_alias_then_deleted);
    define_test!(test_delete_aliases_when_multiple_aliases_then_deleted);
    define_test!(test_delete_aliases_when_non_existent_then_fail);
    define_test!(test_delete_shortcut_when_exists_then_deleted);
    define_test!(test_delete_shortcut_when_not_existent_then_ok);
    define_test!(test_get_default_fallback_shortcut_when_exists_then_ok);
    define_test!(test_get_default_fallback_shortcut_when_non_existent_then_fail);
    define_test!(test_set_default_fallback_shortcut_when_invalid_shortcut_then_fail);
    define_test!(test_set_default_fallback_shortcut_when_non_existent_then_ok);
    define_test!(test_set_default_fallback_shortcut_when_exists_then_updated);
    define_test!(test_set_default_fallback_when_not_fallback_then_fail);
    define_test!(test_get_shortcut_by_uid_when_exists_then_ok);
    define_test!(test_get_shortcut_by_uid_when_non_existent_then_fail);
    define_test!(test_get_shortcut_by_alias_when_single_alias_exists_then_ok);
    define_test!(test_get_shortcut_by_alias_when_multiple_aliases_exist_then_ok);
    define_test!(test_get_shortcut_by_alias_when_non_existent_then_fail);
    define_test!(test_list_shortcuts_when_no_page_size_token_then_fail);
    define_test!(test_list_shortcuts_when_invalid_token_then_fail);
    define_test!(test_list_shortcuts_when_no_shortcuts_then_empty);
    define_test!(test_list_shortcuts_when_one_shortcut_then_ok);
    define_test!(test_list_shortcuts_when_multiple_shortcuts_then_ok);
    define_test!(test_update_shortcut_when_uid_non_existent_then_fail);
    define_test!(test_update_shortcut_when_nothing_updated_then_matches);
    define_test!(test_update_shortcut_when_updated_then_matches);
    define_test!(test_update_shortcut_when_set_is_fallback_false_for_default_then_fails);
    define_test!(test_update_shortcut_when_set_non_unique_url_then_fails);
}
