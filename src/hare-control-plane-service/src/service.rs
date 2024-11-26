use std::fmt::Debug;

use hare_control_plane_model::server::HareControlPlane;
use hare_control_plane_model::ShortcutReferenceAccessExt;
use hare_data_plane_client::error::DataPlaneError;
use hare_data_plane_client::HareDataPlaneClient;

use crate::convert::FromDataPlane;

struct ControlPlaneAliasList<'a>(&'a Vec<hare_control_plane_model::Alias>);

impl<'a> ControlPlaneAliasList<'a> {
    fn get_unique_alias_names(self) -> Vec<&'a str> {
        self.0.iter().map(|alias| alias.name.as_str()).collect::<std::collections::HashSet<_>>().into_iter().collect()
    }
}

#[derive(Debug)]
pub struct HareControlPlaneService<D>
where
    D: HareDataPlaneClient + Send + Sync + 'static,
{
    data_plane_client: D,
}

impl<D: HareDataPlaneClient + Send + Sync + 'static> HareControlPlaneService<D> {
    pub fn new(data_plane_client: D) -> Self {
        HareControlPlaneService { data_plane_client }
    }

    pub fn get_request_id<T>(request: &tonic::Request<T>) -> &str {
        request.extensions().get::<tower_http::request_id::RequestId>().unwrap().header_value().to_str().unwrap()
    }

    /// Get the shortcut reference value from a [`hare_control_plane_model::ShortcutReference`].
    fn get_shortcut_ref_value<'a>(
        shortcut_ref: Option<&'a hare_control_plane_model::ShortcutReference>,
    ) -> hare_data_plane_client::error::DataPlaneResult<&'a hare_control_plane_model::shortcut_reference::Reference>
    {
        match shortcut_ref {
            None => Err(hare_data_plane_client::error::DataPlaneError::InvalidArgument {
                message: "Must provide a shortcut reference".to_string(),
            }),
            Some(shortcut_ref) => {
                shortcut_ref.ok_or_else(|| hare_data_plane_client::error::DataPlaneError::InvalidArgument {
                    message: "Must provide either a shortcut UID or alias".to_string(),
                })
            },
        }
    }

    /// Resolve the shortcut UID from a [`hare_control_plane_model::ShortcutReference`].
    ///
    /// If the ref is an alias the corresponding shortcut is retrieved.
    async fn get_shortcut_uid_from_shortcut_ref<'a>(
        &self,
        shortcut_ref: Option<&'a hare_control_plane_model::ShortcutReference>,
    ) -> hare_data_plane_client::error::DataPlaneResult<std::borrow::Cow<'a, String>> {
        match Self::get_shortcut_ref_value(shortcut_ref)? {
            hare_control_plane_model::shortcut_reference::Reference::Uid(uid) => Ok(std::borrow::Cow::Borrowed(uid)),
            hare_control_plane_model::shortcut_reference::Reference::Alias(alias) => {
                Ok(std::borrow::Cow::Owned(self.data_plane_client.get_shortcut_by_alias(&alias).await?.destination.uid))
            },
        }
    }
}

#[tonic::async_trait]
impl<D: HareDataPlaneClient + std::fmt::Debug + Send + Sync + 'static> HareControlPlane for HareControlPlaneService<D> {
    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        shortcut_ref = request.get_ref().shortcut_ref.as_ref().map(|shortcut_ref| shortcut_ref.get_ref()),
    ))]
    async fn add_aliases_for_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::AddAliasesForShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::AddAliasesForShortcutResponse>, tonic::Status>
    {
        if request.get_ref().aliases.is_empty() {
            return Err(tonic::Status::invalid_argument("Must provide at least one alias to add"));
        }

        let uid = self.get_shortcut_uid_from_shortcut_ref(request.get_ref().shortcut_ref.as_ref()).await?;
        tracing::debug!(shortcut_uid = uid.as_str(), "Resolved shortcut from ref");
        let aliases = ControlPlaneAliasList(&request.get_ref().aliases).get_unique_alias_names();
        let added_aliases =
            self.data_plane_client.add_aliases_for_shortcut(&uid, &aliases).await.inspect_err(|err| {
                tracing::error!(
                    %err,
                    "Failed to add aliases for shortcut",
                );
            })?;
        tracing::info!("Added aliases for shortcut");

        Ok(tonic::Response::new(hare_control_plane_model::AddAliasesForShortcutResponse {
            aliases: added_aliases
                .map(|aliases| aliases.into_iter().map(hare_control_plane_model::Alias::convert).collect())
                .unwrap_or_else(|| Vec::new()),
        }))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        shortcut_url = request.get_ref().shortcut.as_ref().map(|shortcut| &shortcut.url),
    ))]
    async fn create_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::CreateShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::CreateShortcutResponse>, tonic::Status> {
        let request_shortcut = request
            .get_ref()
            .shortcut
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide shortcut to be created"))?;
        if request_shortcut.aliases.is_empty() {
            return Err(tonic::Status::invalid_argument("Shortcuts must have at least one alias"));
        }

        let aliases: Vec<&str> = ControlPlaneAliasList(&request_shortcut.aliases).get_unique_alias_names();
        let committed_shortcut = self
            .data_plane_client
            .create_shortcut(
                &request_shortcut.url,
                request_shortcut.is_fallback.unwrap_or(false),
                &request_shortcut.description,
                &aliases,
            )
            .await
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    "Failed to create shortcut",
                );
            })?;
        tracing::info!(shorcut_uid = committed_shortcut.destination.uid, "Created new shortcut",);

        Ok(tonic::Response::new(hare_control_plane_model::CreateShortcutResponse {
            shortcut: Some(hare_control_plane_model::Shortcut::convert(committed_shortcut)),
        }))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        alias = request.get_ref().alias,
    ))]
    async fn delete_alias_for_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::DeleteAliasForShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let alias = &request.get_ref().alias;
        let shortcut = self.data_plane_client.get_shortcut_by_alias(alias).await?;
        tracing::debug!(shortcut_uid = shortcut.destination.uid, "Resolved shortcut from ref");

        self.data_plane_client
            // The data plane client enforces at least one alias for each shortcut
            .delete_aliases_for_shortcut(&shortcut.destination.uid, &[alias])
            .await
            .inspect_err(|err| {
                tracing::error!(
                    %err,
                    shortcut_uid = shortcut.destination.uid,
                    "Failed to delete alias for shortcut",
                );
            })?;
        tracing::info!(shortcut_uid = shortcut.destination.uid, "Deleted alias for shortcut",);

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        shortcut_ref = request.get_ref().shortcut_ref.as_ref().map(|shortcut_ref| shortcut_ref.get_ref()),
    ))]
    async fn delete_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::DeleteShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let uid = self.get_shortcut_uid_from_shortcut_ref(request.get_ref().shortcut_ref.as_ref()).await?;
        tracing::debug!(shortcut_uid = uid.as_str(), "Resolved shortcut from ref");
        self.data_plane_client.delete_shortcut(&uid).await.inspect_err(|err| {
            tracing::error!(
                %err,
                "Failed to delete shortcut",
            );
        })?;
        tracing::info!("Deleted shortcut");

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
    ))]
    async fn get_default_fallback_shortcut(
        &self,
        request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::GetDefaultFallbackShortcutResponse>, tonic::Status>
    {
        let shortcut = self.data_plane_client.get_default_fallback_shortcut().await.inspect_err(|err| match &err {
            DataPlaneError::NotFound { resource_id: _ } => {
                tracing::error!(request_id = %Self::get_request_id(&request), "No default fallback shortcut defined")
            },
            err => tracing::error!(
                %err,
                "Failed to get default fallback shortcut",
            ),
        })?;

        Ok(tonic::Response::new(hare_control_plane_model::GetDefaultFallbackShortcutResponse {
            shortcut: Some(hare_control_plane_model::Shortcut::convert(shortcut)),
        }))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        shortcut_ref = request.get_ref().shortcut_ref.as_ref().map(|shortcut_ref| shortcut_ref.get_ref()),
    ))]
    async fn get_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::GetShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::GetShortcutResponse>, tonic::Status> {
        let shortcut = match Self::get_shortcut_ref_value(request.get_ref().shortcut_ref.as_ref())? {
            hare_control_plane_model::shortcut_reference::Reference::Uid(uid) => {
                self.data_plane_client.get_shortcut_by_uid(&uid).await
            },
            hare_control_plane_model::shortcut_reference::Reference::Alias(alias) => {
                self.data_plane_client.get_shortcut_by_alias(&alias).await
            },
        }
        .inspect_err(|err| match err {
            DataPlaneError::NotFound { resource_id: _ } => {},
            _ => tracing::error!(
                %err,
                "Failed to get shortcut",
            ),
        })?;
        tracing::debug!(shortcut_uid = shortcut.destination.uid, "Resolved shortcut from ref");

        Ok(tonic::Response::new(hare_control_plane_model::GetShortcutResponse {
            shortcut: Some(hare_control_plane_model::Shortcut::convert(shortcut)),
        }))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        page_size = request.get_ref().pagination.as_ref().map(|pagination| pagination.page_size),
        continuation_token = request.get_ref().pagination.as_ref().map(|pagination| &pagination.continuation_token),
    ))]
    async fn list_shortcuts(
        &self,
        request: tonic::Request<hare_control_plane_model::ListShortcutsRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::ListShortcutsResponse>, tonic::Status> {
        let pagination = request
            .get_ref()
            .pagination
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide pagination information"))?;
        let response = self.data_plane_client.list_shortcuts(pagination).await.map_err(|err| {
            tracing::error!(
                %err,
                "Failed to list shorcuts",
            );
            err
        })?;

        Ok(tonic::Response::new(hare_control_plane_model::ListShortcutsResponse {
            shortcuts: Vec::convert(response.shortcuts),
            pagination_continuation: Some(response.pagination),
        }))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        shortcut_ref = request.get_ref().shortcut_ref.as_ref().map(|shortcut_ref| shortcut_ref.get_ref()),
    ))]
    async fn set_default_fallback_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::SetDefaultFallbackShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let uid = self.get_shortcut_uid_from_shortcut_ref(request.get_ref().shortcut_ref.as_ref()).await.inspect_err(
            |err| match err {
                hare_data_plane_client::error::DataPlaneError::InvalidArgument { message: _ } => {},
                hare_data_plane_client::error::DataPlaneError::NotFound { resource_id: _ } => {},
                _ => tracing::error!(
                    %err,
                    "Failed to get existing shortcut",
                ),
            },
        )?;
        tracing::debug!(shortcut_uid = uid.as_str(), "Resolved shortcut from ref");
        self.data_plane_client.set_default_fallback_shortcut(&uid).await.inspect_err(|err| {
            tracing::error!(
                %err,
                "Failed to set new default fallback shortcut",
            );
        })?;
        tracing::info!("Set new default fallback shortcut");

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument(skip_all, fields(
        headers = ?request.metadata(),
        request_id = Self::get_request_id(&request),
        shortcut_ref = request.get_ref().shortcut_ref.as_ref().map(|shortcut_ref| shortcut_ref.get_ref()),
    ))]
    async fn update_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::UpdateShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::UpdateShortcutResponse>, tonic::Status> {
        let shortcut = if let Some(request_shortcut) = request.get_ref().shortcut.as_ref() {
            let uid = self.get_shortcut_uid_from_shortcut_ref(request.get_ref().shortcut_ref.as_ref()).await?;
            tracing::debug!(shortcut_uid = uid.as_str(), "Resolved shortcut from ref");
            let shortcut = self
                .data_plane_client
                .update_shortcut(
                    &uid,
                    request_shortcut.url.as_deref(),
                    request_shortcut.is_fallback,
                    request_shortcut.description.as_deref(),
                )
                .await
                .inspect_err(|err| {
                    tracing::error!(
                        %err,
                        "Failed to update shortcut",
                    );
                })?;
            tracing::info!("Updated shortcut",);
            Some(shortcut)
        } else {
            None
        };
        Ok(tonic::Response::new(hare_control_plane_model::UpdateShortcutResponse {
            shortcut: shortcut.map(hare_control_plane_model::Shortcut::convert),
        }))
    }
}
