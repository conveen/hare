use hare_control_plane_model::server::HareControlPlane;
use hare_data_plane_client::error::DataPlaneError;
use hare_data_plane_client::HareDataPlaneClient;

use crate::convert::FromDataPlane;

struct ControlPlaneAliasList<'a>(&'a Vec<hare_control_plane_model::Alias>);

impl<'a> From<ControlPlaneAliasList<'a>> for Vec<&'a str> {
    fn from(alias_list: ControlPlaneAliasList<'a>) -> Self {
        alias_list.0.iter().map(|alias| alias.name.as_str()).collect()
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
}

#[tonic::async_trait]
impl<D: HareDataPlaneClient + std::fmt::Debug + Send + Sync + 'static> HareControlPlane for HareControlPlaneService<D> {
    #[tracing::instrument]
    async fn add_aliases_for_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::AddAliasesForShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let request_id = Self::get_request_id(&request);
        let aliases: Vec<&str> = ControlPlaneAliasList(&request.get_ref().aliases).into();
        self.data_plane_client.add_aliases_for_shortcut(&request.get_ref().uid, &aliases).await.map_err(|err| {
            tracing::error!(
                %request_id,
                %err,
                shortcut_uid = request.get_ref().uid,
                aliases = aliases.join(","),
                "Failed to add aliases for shortcut",
            );
            err
        })?;
        tracing::info!(
            %request_id,
            shortcut_uid = request.get_ref().uid,
            "Added aliases for shortcut",
        );

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument]
    async fn create_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::CreateShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::CreateShortcutResponse>, tonic::Status> {
        let request_shortcut = request
            .get_ref()
            .shortcut
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide shortcut to be created"))?;

        let request_id = Self::get_request_id(&request);
        let aliases: Vec<&str> = ControlPlaneAliasList(&request_shortcut.aliases).into();
        let uid = self
            .data_plane_client
            .create_shortcut(
                &request_shortcut.url,
                request_shortcut.is_fallback.unwrap_or(false),
                false,
                &request_shortcut.description,
                &aliases,
            )
            .await
            .map_err(|err| {
                tracing::error!(
                    %request_id,
                    %err,
                    url = &request_shortcut.url,
                    "Failed to create shortcut",
                );
                err
            })?;
        tracing::info!(
            %request_id,
            shorcut_uid = &uid,
            "Created new shortcut",
        );

        Ok(tonic::Response::new(hare_control_plane_model::CreateShortcutResponse { uid }))
    }

    #[tracing::instrument]
    async fn delete_alias_for_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::DeleteAliasForShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let alias = request
            .get_ref()
            .alias
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide the alias to delete"))?;
        let request_id = Self::get_request_id(&request);
        self.data_plane_client
            .delete_aliases_for_shortcut(&request.get_ref().uid, &[alias.name.as_str()])
            .await
            .map_err(|err| {
                tracing::error!(
                    %request_id,
                    %err,
                    shortcut_uid = request.get_ref().uid,
                    alias = alias.name.as_str(),
                    "Failed to delete alias for shortcut",
                );
                err
            })?;
        tracing::info!(
            %request_id,
            shortcut_uid = request.get_ref().uid,
            alias = alias.name.as_str(),
            "Deleted alias for shortcut",
        );

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument]
    async fn delete_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::DeleteShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let request_id = Self::get_request_id(&request);
        self.data_plane_client.delete_shortcut(&request.get_ref().uid).await.map_err(|err| {
            tracing::error!(
                %request_id,
                %err,
                shortcut_uid = request.get_ref().uid,
                "Failed to delete shortcut",
            );
            err
        })?;
        tracing::info!(
            %request_id,
            shortcut_uid = request.get_ref().uid,
            "Deleted shortcut",
        );

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument]
    async fn get_default_fallback_shortcut(
        &self,
        request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::GetDefaultFallbackShortcutResponse>, tonic::Status>
    {
        let request_id = Self::get_request_id(&request);
        let shortcut = self.data_plane_client.get_default_fallback_shortcut().await.inspect_err(|err| match &err {
            DataPlaneError::NotFound { resource_id: _ } => {
                tracing::error!(request_id = %Self::get_request_id(&request), "No default fallback shortcut defined")
            },
            err => tracing::error!(
                %request_id,
                %err,
                "Failed to get default fallback shortcut",
            ),
        })?;

        Ok(tonic::Response::new(hare_control_plane_model::GetDefaultFallbackShortcutResponse {
            shortcut: Some(hare_control_plane_model::Shortcut::convert(shortcut)),
        }))
    }

    #[tracing::instrument]
    async fn get_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::GetShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::GetShortcutResponse>, tonic::Status> {
        let request_id = Self::get_request_id(&request);
        let shortcut = self
            .data_plane_client
            .get_shortcut(
                request.get_ref().uid.as_deref(),
                request.get_ref().alias.as_ref().map(|alias| alias.name.as_str()),
                false,
            )
            .await
            .map_err(|err| {
                tracing::error!(
                    %request_id,
                    %err,
                    shortcut_uid = request.get_ref().uid,
                    "Failed to get shortcut",
                );
                err
            })?;

        Ok(tonic::Response::new(hare_control_plane_model::GetShortcutResponse {
            shortcut: Some(hare_control_plane_model::Shortcut::convert(shortcut)),
        }))
    }

    #[tracing::instrument]
    async fn list_shortcuts(
        &self,
        request: tonic::Request<hare_control_plane_model::ListShortcutsRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::ListShortcutsResponse>, tonic::Status> {
        let pagination = request
            .get_ref()
            .pagination
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide pagination information"))?;
        let request_id = Self::get_request_id(&request);
        let response = self.data_plane_client.list_shortcuts(pagination).await.map_err(|err| {
            tracing::error!(
                %request_id,
                %err,
                pagination.page_size,
                pagination.continuation_token,
                "Failed to list shorcuts",
            );
            err
        })?;

        Ok(tonic::Response::new(hare_control_plane_model::ListShortcutsResponse {
            shortcuts: Vec::convert(response.shortcuts),
            pagination_continuation: Some(response.pagination),
        }))
    }

    #[tracing::instrument]
    async fn set_default_fallback_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::SetDefaultFallbackShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let request_id = Self::get_request_id(&request);
        let shortcut = self
            .data_plane_client
            .get_shortcut(
                request.get_ref().uid.as_deref(),
                request.get_ref().alias.as_ref().map(|alias| alias.name.as_str()),
                false,
            )
            .await
            .inspect_err(|err| match &err {
                DataPlaneError::NotFound { resource_id: _ } => tracing::info!(
                    %request_id,
                    shorcut_uid = request.get_ref().uid,
                    "Attempt to set non-existent shortcut as default fallback",
                ),
                err => tracing::error!(
                    %request_id,
                    %err,
                    shorcut_uid = request.get_ref().uid,
                    "Failed to get existing shortcut for uid",
                ),
            })?;
        self.data_plane_client
            .update_shortcut(Some(&shortcut.destination.uid), None, None, Some(true), Some(true), None)
            .await
            .map_err(|err| {
                tracing::error!(
                    %request_id,
                    %err,
                    shortcut_uid = request.get_ref().uid,
                    "Failed to set shortcut as default fallback",
                );
                err
            })?;
        tracing::info!(
            %request_id,
            shortcut_uid = request.get_ref().uid,
            "Set new default fallback shortcut",
        );

        Ok(tonic::Response::new(()))
    }

    #[tracing::instrument]
    async fn update_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_model::UpdateShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_model::UpdateShortcutResponse>, tonic::Status> {
        let request_id = Self::get_request_id(&request);
        let shortcut = if let Some(request_shortcut) = request.get_ref().shortcut.as_ref() {
            let shortcut = self
                .data_plane_client
                .update_shortcut(
                    request.get_ref().uid.as_deref(),
                    request.get_ref().alias.as_ref().map(|alias| alias.name.as_str()),
                    request_shortcut.url.as_deref(),
                    request_shortcut.is_fallback,
                    None,
                    request_shortcut.description.as_deref(),
                )
                .await
                .map_err(|err| {
                    tracing::error!(
                        %request_id,
                        %err,
                        shortcut_uid = request.get_ref().uid,
                        "Failed to update shortcut",
                    );
                    err
                })?;
            tracing::info!(
                %request_id,
                shortcut_uid = request.get_ref().uid,
                "Updated shortcut",
            );
            shortcut
        } else {
            None
        };
        Ok(tonic::Response::new(hare_control_plane_model::UpdateShortcutResponse {
            shortcut: shortcut.map(hare_control_plane_model::Shortcut::convert),
        }))
    }
}
