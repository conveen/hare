/// Destination that has not been committed to the data plane.
#[derive(Debug, PartialEq)]
pub struct UncommittedDestination {
    /// THe URL (with parameters) to redirect to.
    /// Must be an HTTP URL compliant with [RFC-1738](https://www.rfc-editor.org/rfc/rfc1738#section-3.3).
    pub url: String,
    /// Whether the destination is a fallback.
    pub is_fallback: bool,
    /// Whether the destination is the default fallback.
    pub is_default_fallback: bool,
    /// A description of the URL (and parameters).
    pub description: String,
}

impl UncommittedDestination {
    /// Create a new [`UncommittedDestination`].
    ///
    /// [num_params](struct@UncommittedDestination#structfield.num_params) is parsed from the URL and cannot be set manually.
    pub fn new<S: Into<String>>(url: S, is_fallback: bool, is_default_fallback: bool, description: S) -> Self {
        UncommittedDestination { url: url.into(), is_fallback, is_default_fallback, description: description.into() }
    }
}

/// Destination that has already been committed to the data plane.
#[derive(Debug, PartialEq)]
pub struct CommittedDestination {
    /// The unique ID for the destination.
    pub uid: String,
    /// The URL (with parameters) to redirect to.
    /// Must be an HTTP URL compliant with [RFC-1738](https://www.rfc-editor.org/rfc/rfc1738#section-3.3).
    pub url: String,
    /// The number of parameters in the URL.
    pub num_params: i64,
    /// Whether the destination is a fallback.
    pub is_fallback: bool,
    /// Whether the destination is the default fallback.
    pub is_default_fallback: bool,
    /// A description of the URL (and parameters).
    pub description: String,
}

/// Alias that has not been committed to the data plane.
#[derive(Debug, PartialEq)]
pub struct UncommittedAlias {
    /// The alias name.
    pub name: String,
}

impl UncommittedAlias {
    fn new<S: Into<String>>(name: S) -> Self {
        UncommittedAlias { name: name.into() }
    }
}

impl From<&str> for UncommittedAlias {
    fn from(name: &str) -> Self {
        UncommittedAlias::new(name)
    }
}

impl From<String> for UncommittedAlias {
    fn from(name: String) -> Self {
        UncommittedAlias::new(name)
    }
}

/// Alias that has already been committed to the data plane.
#[derive(Debug, PartialEq)]
pub struct CommittedAlias {
    // The unique ID of the associated [destination](struct@CommittedDestination).
    pub destination_uid: Option<String>,
    // The unique ID for the alias.
    pub uid: Option<String>,
    /// The alias name.
    pub name: String,
}

/// Shortcut that has already been committed to the data plane.
#[derive(Debug, PartialEq)]
pub struct CommittedShortcut {
    /// The shortcut destination.
    pub destination: CommittedDestination,
    /// The shortcut aliases.
    pub aliases: Vec<CommittedAlias>,
}

// TODO: Remove this type and refactor to Vec<CommittedShortcut>
/// List of shortcuts that have already been committed to the data plane.
#[derive(Debug, PartialEq)]
pub struct CommittedShortcutList {
    /// The list of shortcuts.
    pub shortcuts: Vec<CommittedShortcut>,
}
