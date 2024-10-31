use hare_data_plane_client::model::{CommittedAlias, CommittedShortcut, CommittedShortcutList};

/// Cannot implement From
pub(crate) trait FromDataPlane<D> {
    fn convert(data_plane_struct: D) -> Self;
}

impl FromDataPlane<CommittedAlias> for hare_control_plane_model::Alias {
    fn convert(committed_alias: CommittedAlias) -> Self {
        hare_control_plane_model::Alias { uid: committed_alias.uid, name: committed_alias.name }
    }
}

impl FromDataPlane<CommittedShortcut> for hare_control_plane_model::Shortcut {
    fn convert(committed_shortcut: CommittedShortcut) -> Self {
        hare_control_plane_model::Shortcut {
            uid: committed_shortcut.destination.uid,
            url: committed_shortcut.destination.url,
            num_params: committed_shortcut.destination.num_params as i32,
            is_fallback: committed_shortcut.destination.is_fallback,
            is_default_fallback: committed_shortcut.destination.is_default_fallback,
            description: committed_shortcut.destination.description,
            aliases: committed_shortcut.aliases.into_iter().map(hare_control_plane_model::Alias::convert).collect(),
        }
    }
}

impl FromDataPlane<CommittedShortcutList> for Vec<hare_control_plane_model::Shortcut> {
    fn convert(committed_shortcut_list: CommittedShortcutList) -> Self {
        committed_shortcut_list
            .shortcuts
            .into_iter()
            .map(|committed_shortcut| hare_control_plane_model::Shortcut {
                uid: committed_shortcut.destination.uid,
                url: committed_shortcut.destination.url,
                num_params: committed_shortcut.destination.num_params as i32,
                is_fallback: committed_shortcut.destination.is_fallback,
                is_default_fallback: committed_shortcut.destination.is_default_fallback,
                description: committed_shortcut.destination.description,
                aliases: committed_shortcut
                    .aliases
                    .into_iter()
                    .map(|committed_alias| hare_control_plane_model::Alias {
                        uid: committed_alias.uid,
                        name: committed_alias.name,
                    })
                    .collect(),
            })
            .collect()
    }
}
