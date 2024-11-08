use hare_common_model::common;

pub use control_plane::*;

pub trait ShortcutReferenceAccessExt {
    /// Get a ref to the shortcut reference value if provided.
    fn get_ref(&self) -> Option<&str>;
    /// Get a ref to the shortcut reference ([`control_plane::shortcut_reference::Reference`]) if provided, else map to [`Result::Err`].
    fn ok_or_else<E, F>(&self, err: F) -> Result<&control_plane::shortcut_reference::Reference, E>
    where
        F: FnOnce() -> E;
}

impl ShortcutReferenceAccessExt for control_plane::ShortcutReference {
    fn get_ref(&self) -> Option<&str> {
        match self.reference.as_ref() {
            None => None,
            Some(reference) => match reference {
                shortcut_reference::Reference::Uid(uid) => Some(uid.as_str()),
                shortcut_reference::Reference::Alias(alias) => Some(alias.as_str()),
            },
        }
    }

    fn ok_or_else<E, F>(&self, err: F) -> Result<&control_plane::shortcut_reference::Reference, E>
    where
        F: FnOnce() -> E,
    {
        match self.reference.as_ref() {
            None => Err(err()),
            Some(reference) => Ok(reference),
        }
    }
}

pub mod client {
    pub use super::control_plane::hare_control_plane_client::HareControlPlaneClient;
}

pub mod server {
    pub use super::control_plane::hare_control_plane_server::{HareControlPlane, HareControlPlaneServer};
}

mod control_plane {
    tonic::include_proto!("app.conveen.hare.control_plane");
}
