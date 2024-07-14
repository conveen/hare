use hare_control_plane_model::app::conveen::hare::control_plane as hare_control_plane_types;
use hare_control_plane_model::app::conveen::hare::control_plane::hare_control_plane_server::HareControlPlane;

use hare_data_plane_client::HareDataPlaneClient;

use crate::convert::FromDataPlane;

struct ControlPlaneAliasList<'a>(&'a Vec<hare_control_plane_types::Alias>);

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
}

#[tonic::async_trait]
impl<D: HareDataPlaneClient + Send + Sync + 'static> HareControlPlane for HareControlPlaneService<D> {
    async fn add_aliases_for_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::AddAliasesForShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let aliases: Vec<&str> = ControlPlaneAliasList(&request.get_ref().aliases).into();
        self.data_plane_client.add_aliases_for_shortcut(&request.get_ref().uid, &aliases).await?;

        Ok(tonic::Response::new(()))
    }

    async fn create_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::CreateShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_types::CreateShortcutResponse>, tonic::Status> {
        let request_shortcut = request
            .get_ref()
            .shortcut
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide shortcut to be created"))?;

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
            .await?;

        Ok(tonic::Response::new(hare_control_plane_types::CreateShortcutResponse { uid }))
    }

    async fn delete_alias_for_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::DeleteAliasForShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let alias = request
            .get_ref()
            .alias
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide the alias to delete"))?;
        self.data_plane_client.delete_aliases_for_shortcut(&request.get_ref().uid, &[alias.name.as_str()]).await?;

        Ok(tonic::Response::new(()))
    }

    async fn delete_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::DeleteShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        self.data_plane_client.delete_shortcut(&request.get_ref().uid).await?;

        Ok(tonic::Response::new(()))
    }

    async fn get_default_fallback_shortcut(
        &self,
        _request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_types::GetDefaultFallbackShortcutResponse>, tonic::Status>
    {
        let shortcut = self.data_plane_client.get_default_fallback_shortcut().await?;

        Ok(tonic::Response::new(hare_control_plane_types::GetDefaultFallbackShortcutResponse {
            shortcut: Some(hare_control_plane_types::Shortcut::convert(shortcut)),
        }))
    }

    async fn get_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::GetShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_types::GetShortcutResponse>, tonic::Status> {
        let shortcut = self
            .data_plane_client
            .get_shortcut(
                request.get_ref().uid.as_deref(),
                request.get_ref().alias.as_ref().map(|alias| alias.name.as_str()),
                false,
            )
            .await?;

        Ok(tonic::Response::new(hare_control_plane_types::GetShortcutResponse {
            shortcut: Some(hare_control_plane_types::Shortcut::convert(shortcut)),
        }))
    }

    async fn list_shortcuts(
        &self,
        request: tonic::Request<hare_control_plane_types::ListShortcutsRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_types::ListShortcutsResponse>, tonic::Status> {
        let pagination = request
            .get_ref()
            .pagination
            .as_ref()
            .ok_or_else(|| tonic::Status::invalid_argument("Must provide pagination information"))?;
        let response = self.data_plane_client.list_shortcuts(pagination).await?;

        Ok(tonic::Response::new(hare_control_plane_types::ListShortcutsResponse {
            shortcuts: Some(hare_control_plane_types::ShortcutList::convert(response.shortcuts)),
            pagination_continuation: Some(response.pagination),
        }))
    }

    async fn set_default_fallback_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::SetDefaultFallbackShortcutRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        let shortcut = self
            .data_plane_client
            .get_shortcut(
                request.get_ref().uid.as_deref(),
                request.get_ref().alias.as_ref().map(|alias| alias.name.as_str()),
                false,
            )
            .await?;
        self.data_plane_client
            .update_shortcut(Some(&shortcut.destination.uid), None, None, Some(true), Some(true), None)
            .await?;

        Ok(tonic::Response::new(()))
    }

    async fn update_shortcut(
        &self,
        request: tonic::Request<hare_control_plane_types::UpdateShortcutRequest>,
    ) -> std::result::Result<tonic::Response<hare_control_plane_types::UpdateShortcutResponse>, tonic::Status> {
        let shortcut = if let Some(request_shortcut) = request.get_ref().shortcut.as_ref() {
            self.data_plane_client
                .update_shortcut(
                    request.get_ref().uid.as_deref(),
                    request.get_ref().alias.as_ref().map(|alias| alias.name.as_str()),
                    request_shortcut.url.as_deref(),
                    request_shortcut.is_fallback,
                    None,
                    request_shortcut.description.as_deref(),
                )
                .await?
        } else {
            None
        };
        Ok(tonic::Response::new(hare_control_plane_types::UpdateShortcutResponse {
            shortcut: shortcut.map(hare_control_plane_types::Shortcut::convert),
        }))
    }
}
