use hare_common_model::pagination::PaginationRequest;
use hare_data_plane_client::{
    error::{DataPlaneError, DataPlaneResult},
    sqlite::HareDataPlaneSqlite,
    HareDataPlaneClient,
};

#[sqlx::test]
fn test_when_bootstrap_then_ok() -> DataPlaneResult<()> {
    let database_url = std::env::var("DATABASE_URL").expect("Must provide DATABASE_URL environment variable");
    let client = HareDataPlaneSqlite::try_from_url(database_url).await?;
    client.bootstrap().await?;
    Ok(())
}

async fn create_ddg_shortcut(client: &HareDataPlaneSqlite) -> DataPlaneResult<String> {
    client.create_shortcut("https://duckduckgo.com/?q={}", true, true, "DuckDuckGo search", &["d", "duckduckgo"]).await
}

#[sqlx::test]
async fn test_add_aliases_for_shortcut_when_add_single_alias_then_exists(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    client.add_aliases_for_shortcut(&uid.as_str(), &["ddg"]).await?;
    let shortcut = client.get_shortcut_by_uid(&uid.as_str()).await?;
    assert!(shortcut.aliases.into_iter().any(|alias| alias.destination_uid.unwrap() == uid && alias.name == "ddg"));
    Ok(())
}

#[sqlx::test]
async fn test_add_aliases_for_shortcut_when_add_multiple_distinct_aliases_then_exists(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    client.add_aliases_for_shortcut(&uid, &["ddg", "ddgs"]).await?;
    let shortcut = client.get_shortcut_by_uid(&uid.as_str()).await?;
    assert!(
        shortcut
            .aliases
            .into_iter()
            .filter(|alias| alias.destination_uid.as_ref().unwrap() == &uid
                && (alias.name == "ddg" || alias.name == "ddgs"))
            .collect::<Vec<_>>()
            .len()
            == 2
    );
    Ok(())
}

#[sqlx::test]
async fn test_add_aliases_for_shortcut_when_add_multiple_aliases_with_dupes_then_unique(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    client.add_aliases_for_shortcut(&uid, &["ddg", "ddg"]).await?;
    let shortcut = client.get_shortcut_by_uid(&uid.as_str()).await?;
    let mut aliases = shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>();
    aliases.sort();
    assert_eq!(vec!["d", "ddg", "duckduckgo"], aliases,);
    Ok(())
}

#[sqlx::test]
async fn test_add_aliases_for_shortcut_when_add_to_non_existent_shortcut_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.add_aliases_for_shortcut("NonExistentShortcut", &["a", "b"]).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => {
            assert_eq!("NonExistentShortcut", resource_id)
        },
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_add_aliases_for_shortcut_when_add_existing_alias_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    match client.add_aliases_for_shortcut(&uid, &["d"]).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("d", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_add_aliases_for_shortcut_when_add_multiple_one_existing_alias_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    match client.add_aliases_for_shortcut(&uid, &["ddg", "ddgs", "d"]).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("d", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_create_shortcut_when_invalid_url_then_fail(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client
        .create_shortcut("ftp://transfer.example.com", false, false, "FTP transfer for example.com", &["transfer"])
        .await
        .err()
        .unwrap()
    {
        DataPlaneError::InvalidUrl { message } => assert_eq!("invalid scheme ftp", message),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_create_shortcut_when_valid_url_then_exists(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let test_cases = vec![
        hare_data_plane_client::model::UncommittedDestination {
            url: "https://en.wikipedia.org/wiki/Main_Page".to_string(),
            is_fallback: false,
            is_default_fallback: false,
            description: "Wikipedia main page".to_string(),
        },
        hare_data_plane_client::model::UncommittedDestination {
            url: "https://en.wikipedia.org/w/index.php?search={}".to_string(),
            is_fallback: true,
            is_default_fallback: false,
            description: "Wikipedia search".to_string(),
        },
        hare_data_plane_client::model::UncommittedDestination {
            url: "https://en.wikipedia.org/w/index.php?search={}&title={}".to_string(),
            is_fallback: true,
            is_default_fallback: false,
            description: "Wikipedia search".to_string(),
        },
    ]
    .into_iter()
    .zip(vec!["wm", "w", "ws"])
    .zip(vec![0, 1, 1])
    .collect::<Vec<_>>();
    for test_case in test_cases.into_iter() {
        let uid = client
            .create_shortcut(
                &test_case.0 .0.url,
                test_case.0 .0.is_fallback,
                test_case.0 .0.is_default_fallback,
                &test_case.0 .0.description,
                &[test_case.0 .1],
            )
            .await?;
        let shortcut = client.get_shortcut_by_uid(&uid).await?;
        assert_eq!(test_case.0 .0.url, shortcut.destination.url);
        assert_eq!(test_case.0 .0.is_fallback, shortcut.destination.is_fallback);
        assert_eq!(test_case.0 .0.is_default_fallback, shortcut.destination.is_default_fallback);
        assert_eq!(test_case.0 .0.description, shortcut.destination.description);
        assert_eq!(test_case.1, shortcut.destination.num_params);
        assert_eq!(vec![test_case.0 .1], shortcut.aliases.iter().map(|alias| alias.name.as_str()).collect::<Vec<_>>());
    }
    Ok(())
}

#[sqlx::test]
async fn test_create_shortcut_when_url_already_exist_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    create_ddg_shortcut(&client).await?;
    match create_ddg_shortcut(&client).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("https://duckduckgo.com/?q={}", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_create_shortcut_when_empty_aliases_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.create_shortcut("https://wikipedia.org", false, false, "Wikipedia main page", &[]).await.err().unwrap()
    {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_delete_aliases_when_empty_aliases_then_fail(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    assert_eq!(None, client.delete_aliases_for_shortcut(&uid, &[]).await?);
    Ok(())
}

#[sqlx::test]
async fn test_delete_aliases_when_non_existent_shortcut_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.delete_aliases_for_shortcut("NonExistentShortcut", &["d"]).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_delete_aliases_when_single_alias_one_existing_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = client.create_shortcut("https://wikipedia.org", false, false, "Wikipedia main page", &["w"]).await?;
    match client.delete_aliases_for_shortcut(&uid, &["w"]).await.err().unwrap() {
        DataPlaneError::FailedPrecondition { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_delete_aliases_when_more_than_existing_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid =
        client.create_shortcut("https://wikipedia.org", false, false, "Wikipedia main page", &["w", "wp"]).await?;
    match client.delete_aliases_for_shortcut(&uid, &["w", "wp", "wikipedia"]).await.err().unwrap() {
        DataPlaneError::FailedPrecondition { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_delete_aliases_when_one_alias_then_deleted(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let deleted_aliases = client.delete_aliases_for_shortcut(&uid, &["d"]).await?.unwrap();
    assert_eq!(vec!["d"], deleted_aliases);
    let shortcut = client.get_shortcut_by_uid(&uid).await?;
    assert_eq!(vec!["duckduckgo"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

#[sqlx::test]
async fn test_delete_aliases_when_multiple_aliases_then_deleted(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = client
        .create_shortcut("https://wikipedia.org", false, false, "Wikipedia main page", &["w", "wp", "wikipedia"])
        .await?;
    let mut deleted_aliases = client.delete_aliases_for_shortcut(&uid, &["w", "wp"]).await?.unwrap();
    deleted_aliases.sort();
    assert_eq!(vec!["w", "wp"], deleted_aliases);
    let shortcut = client.get_shortcut_by_uid(&uid).await?;
    assert_eq!(vec!["wikipedia"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

#[sqlx::test]
async fn test_delete_aliases_when_non_existent_then_ok(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    assert_eq!(Vec::<String>::new(), client.delete_aliases_for_shortcut(&uid, &["w"]).await?.unwrap());
    Ok(())
}

#[sqlx::test]
async fn test_delete_shortcut_when_exists_then_deleted(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    client.delete_shortcut(&uid).await?;
    match client.get_shortcut_by_uid(&uid).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!(uid, resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_delete_shortcut_when_not_existent_then_ok(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    assert_eq!((), client.delete_shortcut("NonExistentShortcut").await.unwrap());
    Ok(())
}

#[sqlx::test]
async fn test_get_default_fallback_shortcut_when_exists_then_ok(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let default_fallback_shortcut = client.get_default_fallback_shortcut().await?;
    assert_eq!(uid, default_fallback_shortcut.destination.uid);
    Ok(())
}

#[sqlx::test]
async fn test_get_default_fallback_shortcut_when_non_existent_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.get_default_fallback_shortcut().await.err().unwrap() {
        DataPlaneError::NotFound { resource_id: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_get_shortcut_by_uid_when_exists_then_ok(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let shortcut = client.get_shortcut_by_uid(&uid).await?;
    assert_eq!(uid, shortcut.destination.uid);
    assert_eq!(vec!["d", "duckduckgo"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

#[sqlx::test]
async fn test_get_shortcut_by_uid_when_non_existent_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.get_shortcut_by_uid("NonExistentShortcut").await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut".to_string(), resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_get_shortcut_by_alias_when_single_alias_exists_then_ok(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = client.create_shortcut("https://duckduckgo.com/?q={}", true, true, "DuckDuckGo search", &["d"]).await?;
    let shortcut = client.get_shortcut_by_alias("d").await?;
    assert_eq!(uid, shortcut.destination.uid);
    assert_eq!(vec!["d"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

#[sqlx::test]
async fn test_get_shortcut_by_alias_when_multiple_aliases_exist_then_ok(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let shortcut = client.get_shortcut_by_alias("duckduckgo").await?;
    assert_eq!(uid, shortcut.destination.uid);
    assert_eq!(vec!["d", "duckduckgo"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

#[sqlx::test]
async fn test_get_shortcut_by_alias_when_non_existent_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.get_shortcut_by_alias("NonExistentAlias").await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentAlias", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_list_shortcuts_when_no_page_size_token_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: None }).await.err().unwrap() {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_list_shortcuts_when_invalid_token_then_fail(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client
        .list_shortcuts(&PaginationRequest { continuation_token: Some("InvalidToken".to_string()), page_size: None })
        .await
        .err()
        .unwrap()
    {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_list_shortcuts_when_no_shortcuts_then_empty(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(10) }).await?;
    assert_eq!(0, response.shortcuts.shortcuts.len());
    assert_eq!(None, response.pagination.next_continuation_token);
    Ok(())
}

#[sqlx::test]
async fn test_list_shortcuts_when_one_shortcut_then_ok(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(10) }).await?;
    assert_eq!(1, response.shortcuts.shortcuts.len());
    assert!(response.pagination.next_continuation_token.is_some());
    assert_eq!(uid, response.shortcuts.shortcuts.get(0).unwrap().destination.uid);
    let response = client
        .list_shortcuts(&PaginationRequest {
            continuation_token: response.pagination.next_continuation_token,
            page_size: Some(10),
        })
        .await?;
    assert_eq!(0, response.shortcuts.shortcuts.len());
    assert_eq!(None, response.pagination.next_continuation_token);
    Ok(())
}

#[sqlx::test]
async fn test_list_shortcuts_when_multiple_shortcuts_then_ok(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    client
        .create_shortcut("https://duckduckgo.com/?q={}", true, true, "DuckDuckGo search", &["d", "duckduckgo"])
        .await?;
    client.create_shortcut("https://google.com/search?q={}", true, false, "Google search", &["g", "google"]).await?;
    client.create_shortcut("https://wikipedia.org", false, false, "Wikipedia main page", &["wp"]).await?;

    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(2) }).await?;
    assert_eq!(2, response.shortcuts.shortcuts.len());
    assert!(response.pagination.next_continuation_token.is_some());
    let response = client
        .list_shortcuts(&PaginationRequest {
            continuation_token: response.pagination.next_continuation_token,
            page_size: Some(10),
        })
        .await?;
    assert_eq!(1, response.shortcuts.shortcuts.len());
    assert!(response.pagination.next_continuation_token.is_some());
    let response = client
        .list_shortcuts(&PaginationRequest {
            continuation_token: response.pagination.next_continuation_token,
            page_size: Some(10),
        })
        .await?;
    assert_eq!(0, response.shortcuts.shortcuts.len());
    assert_eq!(None, response.pagination.next_continuation_token);

    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(10) }).await?;
    assert_eq!(3, response.shortcuts.shortcuts.len());
    assert!(response.pagination.next_continuation_token.is_some());
    let response = client
        .list_shortcuts(&PaginationRequest {
            continuation_token: response.pagination.next_continuation_token,
            page_size: None,
        })
        .await?;
    assert_eq!(0, response.shortcuts.shortcuts.len());
    assert_eq!(None, response.pagination.next_continuation_token);
    Ok(())
}

#[sqlx::test]
async fn test_update_shortcut_when_uid_alias_none_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.update_shortcut(None, None, None, None, None, None).await.err().unwrap() {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

#[sqlx::test]
async fn test_update_shortcut_when_uid_non_existent_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.update_shortcut(Some("NonExistentShortcut"), None, None, None, None, None).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_update_shortcut_when_alias_non_existent_then_fail(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    match client.update_shortcut(None, Some("d"), None, None, None, None).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("d", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

#[sqlx::test]
async fn test_update_shortcut_when_nothing_updated_then_matches(
    connection: sqlx::Pool<sqlx::Sqlite>,
) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let original = client.get_shortcut_by_uid(&uid).await?;
    let updated = client.update_shortcut(Some(&uid), None, None, None, None, None).await?.unwrap();
    assert_eq!(original.destination, updated.destination);
    assert_eq!(original.aliases, updated.aliases);
    Ok(())
}

#[sqlx::test]
async fn test_update_shortcut_when_updated_then_matches(connection: sqlx::Pool<sqlx::Sqlite>) -> DataPlaneResult<()> {
    let client = HareDataPlaneSqlite::from_connection(connection);
    let uid = create_ddg_shortcut(&client).await?;
    let original = client.get_shortcut_by_uid(&uid).await?;
    let updated = client
        .update_shortcut(Some(&uid), None, None, None, None, Some("New description for shortcut"))
        .await?
        .unwrap();
    assert_eq!("DuckDuckGo search", original.destination.description);
    assert_eq!("New description for shortcut", updated.destination.description);
    assert_eq!(original.aliases, updated.aliases);
    Ok(())
}
