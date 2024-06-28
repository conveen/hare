use hare_control_plane_model::app::conveen::hare::control_plane as hare_control_plane_types;
use hare_data_plane_client::model::{CommittedAlias, CommittedShortcut, CommittedShortcutList};

/// Cannot implement From
pub(crate) trait FromDataPlane<D> {
    fn convert(data_plane_struct: D) -> Self;
}

impl FromDataPlane<CommittedAlias> for hare_control_plane_types::Alias {
    fn convert(committed_alias: CommittedAlias) -> Self {
        hare_control_plane_types::Alias { uid: committed_alias.uid, name: committed_alias.name }
    }
}

impl FromDataPlane<CommittedShortcut> for hare_control_plane_types::Shortcut {
    fn convert(committed_shortcut: CommittedShortcut) -> Self {
        hare_control_plane_types::Shortcut {
            uid: committed_shortcut.destination.uid,
            url: committed_shortcut.destination.url,
            num_params: committed_shortcut.destination.num_params as i32,
            is_fallback: committed_shortcut.destination.is_fallback,
            is_default_fallback: committed_shortcut.destination.is_default_fallback,
            description: committed_shortcut.destination.description,
            aliases: committed_shortcut.aliases.into_iter().map(hare_control_plane_types::Alias::convert).collect(),
        }
    }
}

impl FromDataPlane<CommittedShortcutList> for hare_control_plane_types::ShortcutList {
    fn convert(shortcut_list: CommittedShortcutList) -> Self {
        hare_control_plane_types::ShortcutList {
            shortcuts: shortcut_list.shortcuts.into_iter().map(hare_control_plane_types::Shortcut::convert).collect(),
        }
    }
}
