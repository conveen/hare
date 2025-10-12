use hare_common_model::pagination::PaginationRequest;
use hare_data_plane_client::{
    error::{DataPlaneError, DataPlaneResult},
    model, HareDataPlaneClient,
};

async fn create_ddg_shortcut<D: HareDataPlaneClient + Sync>(client: &D) -> DataPlaneResult<model::CommittedShortcut> {
    let mut shortcut =
        client.create_shortcut("https://duckduckgo.com/?q={}", true, "DuckDuckGo search", &["d", "duckduckgo"]).await?;
    client.set_default_fallback_shortcut(&shortcut.destination.uid).await?;
    shortcut.destination.is_default_fallback = true;
    Ok(shortcut)
}

pub async fn test_when_bootstrap_then_ok<D: HareDataPlaneClient + Sync>(client: D) -> DataPlaneResult<()> {
    client.bootstrap().await?;
    Ok(())
}

pub async fn test_check_health_connection_then_ok<D: HareDataPlaneClient + Sync>(client: &D) -> DataPlaneResult<()> {
    client.check_health().await?;
    Ok(())
}

pub async fn test_add_aliases_for_shortcut_when_add_single_alias_then_exists<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let uid = shortcut.destination.uid;
    client.add_aliases_for_shortcut(&uid, &["ddg"]).await?;
    let shortcut = client.get_shortcut_by_uid(&uid).await?;
    assert!(
        shortcut
            .aliases
            .into_iter()
            .any(|alias| (alias.destination_uid.is_none() || alias.destination_uid.unwrap() == uid)
                && alias.name == "ddg")
    );
    Ok(())
}

pub async fn test_add_aliases_for_shortcut_when_add_multiple_distinct_aliases_then_exists<
    D: HareDataPlaneClient + Sync,
>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let uid = shortcut.destination.uid;
    client.add_aliases_for_shortcut(&uid, &["ddg", "ddgs"]).await?;
    let shortcut = client.get_shortcut_by_uid(&uid.as_str()).await?;
    assert!(
        shortcut
            .aliases
            .into_iter()
            .filter(|alias| (alias.destination_uid.is_none() || alias.destination_uid.as_ref().unwrap() == &uid)
                && (alias.name == "ddg" || alias.name == "ddgs"))
            .collect::<Vec<_>>()
            .len()
            == 2
    );
    Ok(())
}

pub async fn test_add_aliases_for_shortcut_when_add_multiple_aliases_with_dupes_then_fail<
    D: HareDataPlaneClient + Sync,
>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    match client.add_aliases_for_shortcut(&shortcut.destination.uid, &["ddg", "ddg"]).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!(&resource_id, "ddg"),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_add_aliases_for_shortcut_when_add_to_non_existent_shortcut_then_fail<
    D: HareDataPlaneClient + Sync,
>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.add_aliases_for_shortcut("NonExistentShortcut", &["a", "b"]).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => {
            assert_eq!("NonExistentShortcut", resource_id)
        },
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_add_aliases_for_shortcut_when_add_existing_alias_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    match client.add_aliases_for_shortcut(&shortcut.destination.uid, &["d"]).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("d", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_add_aliases_for_shortcut_when_add_multiple_one_existing_alias_then_fail<
    D: HareDataPlaneClient + Sync,
>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    match client.add_aliases_for_shortcut(&shortcut.destination.uid, &["ddg", "ddgs", "d"]).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("d", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_create_shortcut_when_invalid_url_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client
        .create_shortcut("ftp://transfer.example.com", false, "FTP transfer for example.com", &["transfer"])
        .await
        .err()
        .unwrap()
    {
        DataPlaneError::InvalidUrl { message } => assert_eq!("invalid scheme ftp", message),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_create_shortcut_when_valid_url_then_exists<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let test_cases = vec![
        hare_data_plane_client::model::UncommittedDestination {
            url: "https://en.wikipedia.org/wiki/Main_Page".to_string(),
            is_fallback: false,
            description: "Wikipedia main page".to_string(),
        },
        hare_data_plane_client::model::UncommittedDestination {
            url: "https://en.wikipedia.org/w/index.php?search={}".to_string(),
            is_fallback: true,
            description: "Wikipedia search".to_string(),
        },
        hare_data_plane_client::model::UncommittedDestination {
            url: "https://en.wikipedia.org/w/index.php?search={}&title={}".to_string(),
            is_fallback: true,
            description: "Wikipedia search".to_string(),
        },
    ]
    .into_iter()
    .zip(vec!["wm", "w", "ws"])
    .zip(vec![0, 1, 1])
    .collect::<Vec<_>>();
    for test_case in test_cases.into_iter() {
        let shortcut = client
            .create_shortcut(
                &test_case.0 .0.url,
                test_case.0 .0.is_fallback,
                &test_case.0 .0.description,
                &[test_case.0 .1],
            )
            .await?;
        assert_eq!(test_case.0 .0.url, shortcut.destination.url);
        assert_eq!(test_case.0 .0.is_fallback, shortcut.destination.is_fallback);
        assert_eq!(false, shortcut.destination.is_default_fallback);
        assert_eq!(test_case.0 .0.description, shortcut.destination.description);
        assert_eq!(test_case.1, shortcut.destination.num_params);
        assert_eq!(vec![test_case.0 .1], shortcut.aliases.iter().map(|alias| alias.name.as_str()).collect::<Vec<_>>());
    }
    Ok(())
}

pub async fn test_create_shortcut_when_url_already_exist_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    create_ddg_shortcut(client).await?;
    match create_ddg_shortcut(client).await.err().unwrap() {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("https://duckduckgo.com/?q={}", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_create_shortcut_when_empty_aliases_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.create_shortcut("https://wikipedia.org", false, "Wikipedia main page", &[]).await.err().unwrap() {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_delete_aliases_when_empty_aliases_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    assert_eq!(None, client.delete_aliases_for_shortcut(&shortcut.destination.uid, &[]).await?);
    Ok(())
}

pub async fn test_delete_aliases_when_non_existent_shortcut_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.delete_aliases_for_shortcut("NonExistentShortcut", &["d"]).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_delete_aliases_when_single_alias_one_existing_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = client.create_shortcut("https://wikipedia.org", false, "Wikipedia main page", &["w"]).await?;
    match client.delete_aliases_for_shortcut(&shortcut.destination.uid, &["w"]).await.err().unwrap() {
        DataPlaneError::FailedPrecondition { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_delete_aliases_when_more_than_existing_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = client.create_shortcut("https://wikipedia.org", false, "Wikipedia main page", &["w", "wp"]).await?;
    match client.delete_aliases_for_shortcut(&shortcut.destination.uid, &["w", "wp", "wikipedia"]).await.err().unwrap()
    {
        DataPlaneError::FailedPrecondition { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_delete_aliases_when_one_alias_then_deleted<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let deleted_aliases = client.delete_aliases_for_shortcut(&shortcut.destination.uid, &["d"]).await?.unwrap();
    assert_eq!(vec!["d"], deleted_aliases.iter().map(|alias| &alias.name).collect::<Vec<_>>());
    let shortcut = client.get_shortcut_by_uid(&shortcut.destination.uid).await?;
    assert_eq!(vec!["duckduckgo"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

pub async fn test_delete_aliases_when_multiple_aliases_then_deleted<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = client
        .create_shortcut("https://wikipedia.org", false, "Wikipedia main page", &["w", "wp", "wikipedia"])
        .await?;
    let uid = shortcut.destination.uid;
    let mut deleted_aliases = client.delete_aliases_for_shortcut(&uid, &["w", "wp"]).await?.unwrap();
    deleted_aliases.sort();
    assert_eq!(vec!["w", "wp"], deleted_aliases.iter().map(|alias| &alias.name).collect::<Vec<_>>());
    let shortcut = client.get_shortcut_by_uid(&uid).await?;
    assert_eq!(vec!["wikipedia"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

pub async fn test_delete_aliases_when_non_existent_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    match client.delete_aliases_for_shortcut(&shortcut.destination.uid, &["w"]).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("w", &resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_delete_shortcut_when_exists_then_deleted<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let uid = shortcut.destination.uid;
    client.delete_shortcut(&uid).await?;
    match client.get_shortcut_by_uid(&uid).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!(uid, resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_delete_shortcut_when_not_existent_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.delete_shortcut("NonExistentShortcut").await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_get_default_fallback_shortcut_when_exists_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let default_fallback_shortcut = client.get_default_fallback_shortcut().await?;
    assert_eq!(shortcut.destination.uid, default_fallback_shortcut.destination.uid);
    Ok(())
}

pub async fn test_get_default_fallback_shortcut_when_non_existent_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.get_default_fallback_shortcut().await.err().unwrap() {
        DataPlaneError::NotFound { resource_id: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_set_default_fallback_shortcut_when_invalid_shortcut_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.set_default_fallback_shortcut("NonExistentShortcut").await.err().unwrap() {
        DataPlaneError::NotFound { resource_id: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_set_default_fallback_shortcut_when_non_existent_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = client
        .create_shortcut("https://wikipedia.org/w/index.php?search={}", true, "Wikipedia search", &["w", "wikipedia"])
        .await?;
    client.set_default_fallback_shortcut(&shortcut.destination.uid).await?;
    let default_fallback = client.get_default_fallback_shortcut().await?;
    assert_eq!(shortcut.destination.uid, default_fallback.destination.uid);
    Ok(())
}

pub async fn test_set_default_fallback_shortcut_when_exists_then_updated<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    create_ddg_shortcut(client).await?;
    let shortcut = client
        .create_shortcut("https://wikipedia.org/w/index.php?search={}", true, "Wikipedia search", &["w", "wikipedia"])
        .await?;
    client.set_default_fallback_shortcut(&shortcut.destination.uid).await?;
    let new_default_fallback = client.get_default_fallback_shortcut().await?;
    assert_eq!(shortcut.destination.uid, new_default_fallback.destination.uid);
    Ok(())
}

pub async fn test_set_default_fallback_when_not_fallback_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = client
        .create_shortcut("https://wikipedia.org/w/index.php?search={}", false, "Wikipedia search", &["w", "wikipedia"])
        .await?;
    match client.set_default_fallback_shortcut(&shortcut.destination.uid).await.err().unwrap() {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_get_shortcut_by_uid_when_exists_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let uid = shortcut.destination.uid;
    let shortcut = client.get_shortcut_by_uid(&uid).await?;
    assert_eq!(uid, shortcut.destination.uid);
    assert_eq!(vec!["d", "duckduckgo"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

pub async fn test_get_shortcut_by_uid_when_non_existent_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.get_shortcut_by_uid("NonExistentShortcut").await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut".to_string(), resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_get_shortcut_by_alias_when_single_alias_exists_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = client.create_shortcut("https://duckduckgo.com/?q={}", true, "DuckDuckGo search", &["d"]).await?;
    let uid = shortcut.destination.uid;
    let shortcut = client.get_shortcut_by_alias("d").await?;
    assert_eq!(uid, shortcut.destination.uid);
    assert_eq!(vec!["d"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

pub async fn test_get_shortcut_by_alias_when_multiple_aliases_exist_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let uid = shortcut.destination.uid;
    let shortcut = client.get_shortcut_by_alias("duckduckgo").await?;
    assert_eq!(uid, shortcut.destination.uid);
    assert_eq!(vec!["d", "duckduckgo"], shortcut.aliases.into_iter().map(|alias| alias.name).collect::<Vec<_>>());
    Ok(())
}

pub async fn test_get_shortcut_by_alias_when_non_existent_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.get_shortcut_by_alias("NonExistentAlias").await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentAlias", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_list_shortcuts_when_no_page_size_token_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: None }).await.err().unwrap() {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_list_shortcuts_when_invalid_token_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
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

pub async fn test_list_shortcuts_when_no_shortcuts_then_empty<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(10) }).await?;
    assert_eq!(0, response.shortcuts.shortcuts.len());
    assert_eq!(None, response.pagination.next_continuation_token);
    Ok(())
}

pub async fn test_list_shortcuts_when_one_shortcut_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(10) }).await?;
    assert_eq!(1, response.shortcuts.shortcuts.len());
    assert_eq!(shortcut.destination.uid, response.shortcuts.shortcuts.get(0).unwrap().destination.uid);
    if let Some(continuation_token) = response.pagination.next_continuation_token {
        let response = client
            .list_shortcuts(&PaginationRequest { continuation_token: Some(continuation_token), page_size: Some(10) })
            .await?;
        assert_eq!(0, response.shortcuts.shortcuts.len());
        assert_eq!(None, response.pagination.next_continuation_token);
    }
    Ok(())
}

pub async fn test_list_shortcuts_when_multiple_shortcuts_then_ok<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    client.create_shortcut("https://duckduckgo.com/?q={}", true, "DuckDuckGo search", &["d", "duckduckgo"]).await?;
    client.create_shortcut("https://google.com/search?q={}", true, "Google search", &["g", "google"]).await?;
    client.create_shortcut("https://wikipedia.org", false, "Wikipedia main page", &["wp"]).await?;

    let response = client.list_shortcuts(&PaginationRequest { continuation_token: None, page_size: Some(2) }).await?;
    dbg!(&response.shortcuts, &response.pagination);
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

pub async fn test_update_shortcut_when_uid_non_existent_then_fail<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    match client.update_shortcut("NonExistentShortcut", None, None, None).await.err().unwrap() {
        DataPlaneError::NotFound { resource_id } => assert_eq!("NonExistentShortcut", resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}

pub async fn test_update_shortcut_when_nothing_updated_then_matches<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let original = create_ddg_shortcut(client).await?;
    let updated = client.update_shortcut(&original.destination.uid, None, None, None).await?;
    assert_eq!(original.destination, updated.destination);
    assert_eq!(original.aliases, updated.aliases);
    Ok(())
}

pub async fn test_update_shortcut_when_updated_then_matches<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let original = create_ddg_shortcut(client).await?;
    let updated =
        client.update_shortcut(&original.destination.uid, None, None, Some("New description for shortcut")).await?;
    assert_eq!("DuckDuckGo search", original.destination.description);
    assert_eq!("New description for shortcut", updated.destination.description);
    assert_eq!(original.aliases, updated.aliases);
    Ok(())
}

pub async fn test_update_shortcut_when_set_is_fallback_false_for_default_then_fails<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let shortcut = create_ddg_shortcut(client).await?;
    match client.update_shortcut(&shortcut.destination.uid, None, Some(false), None).await.err().unwrap() {
        DataPlaneError::InvalidArgument { message: _ } => Ok(()),
        err => panic!("Unexpected error returned: {:?}", err),
    }
}

pub async fn test_update_shortcut_when_set_non_unique_url_then_fails<D: HareDataPlaneClient + Sync>(
    client: &D,
) -> DataPlaneResult<()> {
    let first_shortcut = create_ddg_shortcut(client).await?;
    let _second_shortcut =
        client.create_shortcut("https://google.com/search?q={}", true, "Google search", &["g", "google"]).await?;
    match client
        .update_shortcut(
            &first_shortcut.destination.uid,
            Some("https://google.com/search?q={}"),
            None,
            Some("Google Search"),
        )
        .await
        .err()
        .unwrap()
    {
        DataPlaneError::AlreadyExists { resource_id } => assert_eq!("https://google.com/search?q={}", &resource_id),
        err => panic!("Unexpected error returned: {:?}", err),
    }
    Ok(())
}
