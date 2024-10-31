use tracing::{debug, error, info};

const BOOTSTRAP_SHORTCUTS: [BootstrapShortcut; 24] = [
    BootstrapShortcut {
        url: "https://duckduckgo.com/?q={}",
        is_fallback: true,
        description: "Search DuckDuckGo",
        aliases: &["d", "ddg", "duckduckgo"],
    },
    BootstrapShortcut {
        url: "https://www.google.com/search?q={}",
        is_fallback: true,
        description: "Search Google",
        aliases: &["g", "google"],
    },
    BootstrapShortcut {
        url: "https://www.bing.com/search?q={}",
        is_fallback: true,
        description: "Search Bing",
        aliases: &["bing"],
    },
    BootstrapShortcut {
        url: "https://en.wikipedia.org/w/index.php?search={}",
        is_fallback: false,
        description: "Search Wikipedia",
        aliases: &["w", "wp", "wikipedia"],
    },
    BootstrapShortcut {
        url: "https://www.reddit.com/r/{}",
        is_fallback: false,
        description: "Reddit - browse to specific subreddit",
        aliases: &["r", "reddit"],
    },
    BootstrapShortcut {
        url: "https://www.reddit.com/search/?q={}",
        is_fallback: false,
        description: "Search Reddit",
        aliases: &["rs", "redditsearch"],
    },
    BootstrapShortcut {
        url: "https://x.com/{}",
        is_fallback: false,
        description: "Twitter (X) - browse to specific user",
        aliases: &["x", "t", "twitter"],
    },
    BootstrapShortcut {
        url: "https://x.com/search?q={}",
        is_fallback: false,
        description: "Search Twitter (X)",
        aliases: &["xs", "ts", "xsearch", "twittersearch"],
    },
    BootstrapShortcut {
        url: "https://www.youtube.com/results?search_query={}",
        is_fallback: false,
        description: "Search YouTube",
        aliases: &["yt", "youtube"],
    },
    BootstrapShortcut {
        url: "https://www.youtube.com/c/{}",
        is_fallback: false,
        description: "YouTube - browse to specific channel",
        aliases: &["ytc", "youtubechannel"],
    },
    BootstrapShortcut {
        url: "https://time.gov/?t=24",
        is_fallback: false,
        description: "National Institute of Standards and Technology (NIST) Official Time",
        aliases: &["time"],
    },
    BootstrapShortcut {
        url: "https://www.nytimes.com",
        is_fallback: false,
        description: "New York Times - main page",
        aliases: &["nyt", "newyorktimes"],
    },
    BootstrapShortcut {
        url: "https://cnn.com/",
        is_fallback: false,
        description: "CNN - main page",
        aliases: &["cnn"],
    },
    BootstrapShortcut {
        url: "https://www.foxnews.com/",
        is_fallback: false,
        description: "Fox News - main page",
        aliases: &["fox", "foxnews"],
    },
    BootstrapShortcut {
        url: "https://apnews.com/",
        is_fallback: false,
        description: "Associated Press News - main page",
        aliases: &["apn", "apnews"],
    },
    BootstrapShortcut {
        url: "https://www.npr.org/",
        is_fallback: false,
        description: "NPR News - main page",
        aliases: &["npr"],
    },
    BootstrapShortcut {
        url: "https://pypi.org/project/{}/",
        is_fallback: false,
        description: "PyPI - browse to specific library",
        aliases: &["pypi"],
    },
    BootstrapShortcut {
        url: "https://pypi.org/search/?q={}",
        is_fallback: false,
        description: "Search PyPI",
        aliases: &["pypis", "pypisearch"],
    },
    BootstrapShortcut {
        url: "https://crates.io/crates/{}",
        is_fallback: false,
        description: "Crates.io - browse to specific crate",
        aliases: &["crate"],
    },
    BootstrapShortcut {
        url: "https://crates.io/search?q={}",
        is_fallback: false,
        description: "Search Crates.io",
        aliases: &["crates", "cratesearch"],
    },
    BootstrapShortcut {
        url: "https://www.npmjs.com/package/{}",
        is_fallback: false,
        description: "NPM - browse to specific package",
        aliases: &["npmp", "npmpackage"],
    },
    BootstrapShortcut {
        url: "https://www.npmjs.com/search?q={}",
        is_fallback: false,
        description: "Search NPM",
        aliases: &["npms", "npmsearch"],
    },
    BootstrapShortcut {
        url: "https://rubygems.org/gems/{}",
        is_fallback: false,
        description: "RubyGems - browse to specific gem",
        aliases: &["gem"],
    },
    BootstrapShortcut {
        url: "https://rubygems.org/search?query={}",
        is_fallback: false,
        description: "Search RubyGems",
        aliases: &["gems", "gemsearch"],
    },
];

struct BootstrapShortcut {
    url: &'static str,
    is_fallback: bool,
    description: &'static str,
    aliases: &'static [&'static str],
}

impl From<&BootstrapShortcut> for hare_control_plane_model::CreateShortcutRequest {
    fn from(shortcut: &BootstrapShortcut) -> Self {
        Self {
            shortcut: Some(hare_control_plane_model::CreateShortcutRequestShortcut {
                url: shortcut.url.to_string(),
                is_fallback: Some(shortcut.is_fallback),
                description: shortcut.description.to_string(),
                aliases: shortcut
                    .aliases
                    .iter()
                    .copied()
                    .map(|alias_name| hare_control_plane_model::Alias { uid: None, name: alias_name.to_string() })
                    .collect(),
            }),
        }
    }
}

/// Arguments for commands that take one of shortcut UID or alias.
#[derive(Clone, Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub(crate) struct ShortcutUidOrAliasArgGroup {
    /// Shortcut UID.
    #[arg(short, long)]
    uid: Option<String>,
    /// Shortcut alias.
    #[arg(short, long)]
    alias: Option<String>,
}

/// Arguments for commands that paginate.
#[derive(Clone, Debug, clap::Args)]
pub(crate) struct PaginationArgs {
    /// Page size.
    #[arg(short, long, default_value_t = 10)]
    page_size: i32,
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct AddAliasesForShortcutArgs {
    /// Shortcut UID.
    uid: String,
    /// Aliases to add.
    #[arg(required = true)]
    aliases: Vec<String>,
}

impl From<&AddAliasesForShortcutArgs> for hare_control_plane_model::AddAliasesForShortcutRequest {
    fn from(args: &AddAliasesForShortcutArgs) -> Self {
        Self {
            uid: args.uid.clone(),
            aliases: args
                .aliases
                .iter()
                .map(|alias_name| hare_control_plane_model::Alias { uid: None, name: alias_name.clone() })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub(crate) struct BootstrapArgs {
    /// Use DuckDuckGo as the default fallback search engine.
    #[arg(short, long, action)]
    duckduckgo: bool,
    /// Use Google as the default fallback search engine.
    #[arg(short, long, action)]
    google: bool,
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct CreateShortcutArgs {
    /// URL template.
    url: String,
    /// Whether the shortcut is a fallback.
    #[arg(short = 'f', long = "fallback", action = clap::ArgAction::SetTrue)]
    is_fallback: bool,
    /// Description.
    description: String,
    /// Aliases for the shortcut.
    #[arg(required = true)]
    aliases: Vec<String>,
}

impl From<&CreateShortcutArgs> for hare_control_plane_model::CreateShortcutRequest {
    fn from(args: &CreateShortcutArgs) -> Self {
        Self {
            shortcut: Some(hare_control_plane_model::CreateShortcutRequestShortcut {
                url: args.url.clone(),
                is_fallback: Some(args.is_fallback),
                description: args.description.clone(),
                aliases: args
                    .aliases
                    .iter()
                    .map(|alias_name| hare_control_plane_model::Alias { uid: None, name: alias_name.clone() })
                    .collect(),
            }),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct DeleteAliasForShortcutArgs {
    /// Shortcut UID.
    uid: String,
    /// Alias to delete.
    alias: String,
}

impl From<&DeleteAliasForShortcutArgs> for hare_control_plane_model::DeleteAliasForShortcutRequest {
    fn from(args: &DeleteAliasForShortcutArgs) -> Self {
        hare_control_plane_model::DeleteAliasForShortcutRequest {
            uid: args.uid.clone(),
            alias: Some(hare_control_plane_model::Alias { uid: None, name: args.alias.clone() }),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct ListShortcutsArgs {
    #[command(flatten)]
    pagination: PaginationArgs,
}

impl From<&ListShortcutsArgs> for hare_control_plane_model::ListShortcutsRequest {
    fn from(args: &ListShortcutsArgs) -> Self {
        hare_control_plane_model::ListShortcutsRequest {
            pagination: Some(hare_common_model::pagination::PaginationRequest {
                continuation_token: None,
                page_size: Some(args.pagination.page_size),
            }),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct DeleteShortcutArgs {
    /// Shortcut UID.
    uid: String,
}

impl From<&DeleteShortcutArgs> for hare_control_plane_model::DeleteShortcutRequest {
    fn from(args: &DeleteShortcutArgs) -> Self {
        hare_control_plane_model::DeleteShortcutRequest { uid: args.uid.clone() }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct GetShortcutArgs {
    #[command(flatten)]
    uid_or_alias: ShortcutUidOrAliasArgGroup,
}

impl From<&GetShortcutArgs> for hare_control_plane_model::GetShortcutRequest {
    fn from(args: &GetShortcutArgs) -> Self {
        hare_control_plane_model::GetShortcutRequest {
            uid: args.uid_or_alias.uid.clone(),
            alias: args
                .uid_or_alias
                .alias
                .as_ref()
                .map(|alias_name| hare_control_plane_model::Alias { uid: None, name: alias_name.clone() }),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct SetDefaultFallbackShortcutArgs {
    #[command(flatten)]
    uid_or_alias: ShortcutUidOrAliasArgGroup,
}

impl From<&SetDefaultFallbackShortcutArgs> for hare_control_plane_model::SetDefaultFallbackShortcutRequest {
    fn from(args: &SetDefaultFallbackShortcutArgs) -> Self {
        hare_control_plane_model::SetDefaultFallbackShortcutRequest {
            uid: args.uid_or_alias.uid.clone(),
            alias: args
                .uid_or_alias
                .alias
                .as_ref()
                .map(|alias_name| hare_control_plane_model::Alias { uid: None, name: alias_name.clone() }),
        }
    }
}

#[derive(Clone, Debug, clap::Args)]
#[group(required = true, multiple = true)]
pub(crate) struct UpdateShortcutAttributesArgGroup {
    /// New shortcut URL.
    #[arg(short = 'U', long)]
    url: Option<String>,
    /// Toggle fallback flag.
    #[arg(short = 'f', long = "fallback", default_missing_value = "None", num_args = 0..=1)]
    is_fallback: Option<bool>,
    /// New description.
    #[arg(short, long)]
    description: Option<String>,
}

#[derive(Clone, Debug, clap::Args)]
pub(crate) struct UpdateShortcutArgs {
    #[command(flatten)]
    uid_or_alias: ShortcutUidOrAliasArgGroup,
    #[command(flatten)]
    attributes: UpdateShortcutAttributesArgGroup,
}

impl From<&UpdateShortcutArgs> for hare_control_plane_model::UpdateShortcutRequest {
    fn from(args: &UpdateShortcutArgs) -> Self {
        hare_control_plane_model::UpdateShortcutRequest {
            uid: args.uid_or_alias.uid.clone(),
            alias: args
                .uid_or_alias
                .alias
                .as_ref()
                .map(|alias_name| hare_control_plane_model::Alias { uid: None, name: alias_name.clone() }),
            shortcut: Some(hare_control_plane_model::UpdateShortcutRequestShortcut {
                url: args.attributes.url.clone(),
                is_fallback: args.attributes.is_fallback,
                description: args.attributes.description.clone(),
            }),
        }
    }
}

/// Control plane commands
#[derive(Clone, Debug, clap::Subcommand)]
pub(crate) enum ControlPlaneCommands {
    /// Add aliases for a shortcut.
    #[command(alias = "aa")]
    AddAliasesForShortcut(AddAliasesForShortcutArgs),
    /// Bootstrap the database with common shortcuts.
    Bootstrap(BootstrapArgs),
    /// Create a new shortcut.
    #[command(alias = "cs")]
    CreateShortcut(CreateShortcutArgs),
    /// Delete an aliases for a shortcut.
    #[command(alias = "da")]
    DeleteAliasForShortcut(DeleteAliasForShortcutArgs),
    /// Delete a shortcut.
    #[command(alias = "ds")]
    DeleteShortcut(DeleteShortcutArgs),
    /// Get the default fallback shortcut.
    #[command(alias = "gd")]
    GetDefaultFallbackShortcut,
    /// Get a shortcut by UID or alias.
    #[command(alias = "gs")]
    GetShortcut(GetShortcutArgs),
    /// List all shortcuts.
    #[command(alias = "ls")]
    ListShortcuts(ListShortcutsArgs),
    /// Set the default fallback shortcut.
    #[command(alias = "sd")]
    SetDefaultFallbackShortcut(SetDefaultFallbackShortcutArgs),
    /// Update one or more fields of a shortcut.
    #[command(alias = "us")]
    UpdateShortcut(UpdateShortcutArgs),
}

impl ControlPlaneCommands {
    /// Create a control plane client from and endpoint URL.
    async fn create_client(
        endpoint: &str,
    ) -> hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel> {
        let endpoint_url: tonic::transport::Uri = endpoint.parse().unwrap_or_else(|err| {
            error!(%err, "Failed to parse endpoint as a valid URL");
            std::process::exit(1);
        });
        debug!(%endpoint_url, "Connecting to endpoint URL");
        hare_control_plane_model::client::HareControlPlaneClient::connect(endpoint_url).await.unwrap_or_else(|err| {
            error!(?err, "Failed to connect to endpoint URL");
            std::process::exit(1);
        })
    }

    /// Extract request ID from an error response ([`tonic::Status`]).
    fn get_request_id_from_err_or_exit(err: &tonic::Status) -> &str {
        err.metadata()
            .get(hare_common_utils::request_id::REQUEST_ID_HEADER_NAME)
            .unwrap_or_else(|| {
                error!("Failed to get request ID from response");
                std::process::exit(1);
            })
            .to_str()
            .unwrap_or_else(|err| {
                error!(%err, "Failed to decode request ID from response");
                std::process::exit(1);
            })
    }

    async fn run_add_aliases_for_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: AddAliasesForShortcutArgs,
    ) {
        client
            .add_aliases_for_shortcut(hare_control_plane_model::AddAliasesForShortcutRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to add aliases for shortcut");
                std::process::exit(1);
            });
        info!(shortcut_uid = args.uid, "Added aliases for shortcut");
    }

    async fn run_bootstrap(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: BootstrapArgs,
    ) {
        for shortcut in BOOTSTRAP_SHORTCUTS.iter() {
            if let Ok(response) = client
                .create_shortcut(hare_control_plane_model::CreateShortcutRequest::from(shortcut))
                .await
                .inspect_err(|err| {
                    let request_id = Self::get_request_id_from_err_or_exit(&err);
                    match err.code() {
                        tonic::Code::AlreadyExists => {
                            info!(shortcut_url = shortcut.url, "Shortcut already exists");
                        },
                        _ => {
                            error!(%request_id, %err, shortcut_url = shortcut.url, "Failed to create shortcut");
                            std::process::exit(1);
                        },
                    }
                })
            {
                info!(shortcut_uid = response.get_ref().uid, shortcut_url = shortcut.url, "Created new shortcut");
            }
        }

        if args.google {
            Self::run_set_default_fallback_shortcut(
                client,
                SetDefaultFallbackShortcutArgs {
                    uid_or_alias: ShortcutUidOrAliasArgGroup { uid: None, alias: Some("google".to_string()) },
                },
            )
            .await;
            info!("Set default fallback shortcut to Google search");
        } else if args.duckduckgo {
            Self::run_set_default_fallback_shortcut(
                client,
                SetDefaultFallbackShortcutArgs {
                    uid_or_alias: ShortcutUidOrAliasArgGroup { uid: None, alias: Some("duckduckgo".to_string()) },
                },
            )
            .await;
            info!("Set default fallback shortcut to DuckDuckGo search");
        }
    }

    async fn run_create_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: CreateShortcutArgs,
    ) {
        let response = client
            .create_shortcut(hare_control_plane_model::CreateShortcutRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to create shortcut");
                std::process::exit(1);
            });
        info!(shortcut_uid = response.get_ref().uid, shortcut_url = args.url, "Created new shortcut");
        println!("{}", serde_json::to_string_pretty(response.get_ref()).unwrap());
    }

    async fn run_delete_alias_for_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: DeleteAliasForShortcutArgs,
    ) {
        client
            .delete_alias_for_shortcut(hare_control_plane_model::DeleteAliasForShortcutRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to delete alias for shortcut");
                std::process::exit(1);
            });
        info!(shortcut_uid = args.uid, "Deleted alias for shortcut");
    }

    async fn run_list_shortcuts(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: ListShortcutsArgs,
    ) {
        let mut all_shortcuts: Vec<hare_control_plane_model::Shortcut> = Vec::new();

        let hare_control_plane_model::ListShortcutsResponse { shortcuts, pagination_continuation } = client
            .list_shortcuts(hare_control_plane_model::ListShortcutsRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to list shortcuts");
                std::process::exit(1);
            })
            .into_inner();
        all_shortcuts.extend(shortcuts);
        let mut continuation_token = match pagination_continuation {
            Some(pagination) => pagination.next_continuation_token,
            None => None,
        };

        while continuation_token.is_some() {
            let hare_control_plane_model::ListShortcutsResponse { shortcuts, pagination_continuation } = client
                .list_shortcuts(hare_control_plane_model::ListShortcutsRequest {
                    pagination: Some(hare_common_model::pagination::PaginationRequest {
                        continuation_token: continuation_token.clone(),
                        page_size: None,
                    }),
                })
                .await
                .unwrap_or_else(|err| {
                    let request_id = Self::get_request_id_from_err_or_exit(&err);
                    error!(%request_id, %err, "Failed to list shortcuts");
                    std::process::exit(1);
                })
                .into_inner();
            all_shortcuts.extend(shortcuts);
            continuation_token = match pagination_continuation {
                Some(pagination) => pagination.next_continuation_token,
                None => None,
            };
        }

        println!("{}", serde_json::to_string_pretty(&all_shortcuts).unwrap());
    }

    async fn run_delete_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: DeleteShortcutArgs,
    ) {
        client.delete_shortcut(hare_control_plane_model::DeleteShortcutRequest::from(&args)).await.unwrap_or_else(
            |err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to delete shortcut");
                std::process::exit(1);
            },
        );
        info!(shortcut_uid = args.uid, "Deleted shortcut");
    }

    async fn run_get_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: GetShortcutArgs,
    ) {
        let response = client
            .get_shortcut(hare_control_plane_model::GetShortcutRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to get shortcut");
                std::process::exit(1);
            });
        println!("{}", serde_json::to_string_pretty(response.get_ref()).unwrap());
    }

    async fn run_set_default_fallback_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: SetDefaultFallbackShortcutArgs,
    ) {
        client
            .set_default_fallback_shortcut(hare_control_plane_model::SetDefaultFallbackShortcutRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to set default fallback shortcut");
                std::process::exit(1);
            });
        info!(
            shortcut_uid_or_alias =
                args.uid_or_alias.uid.as_ref().or_else(|| args.uid_or_alias.alias.as_ref()).unwrap(),
            "Set default fallback shortcut"
        );
    }

    async fn run_update_shortcut(
        client: &mut hare_control_plane_model::client::HareControlPlaneClient<tonic::transport::Channel>,
        args: UpdateShortcutArgs,
    ) {
        let response = client
            .update_shortcut(hare_control_plane_model::UpdateShortcutRequest::from(&args))
            .await
            .unwrap_or_else(|err| {
                let request_id = Self::get_request_id_from_err_or_exit(&err);
                error!(%request_id, %err, "Failed to update shortcut");
                std::process::exit(1);
            });
        info!(
            shortcut_uid_or_alias =
                args.uid_or_alias.uid.as_ref().or_else(|| args.uid_or_alias.alias.as_ref()).unwrap(),
            "Updated shortcut"
        );
        println!("{}", serde_json::to_string_pretty(response.get_ref()).unwrap());
    }

    pub async fn run(self, endpoint: String) {
        let mut client = Self::create_client(endpoint.as_str()).await;
        match self {
            ControlPlaneCommands::AddAliasesForShortcut(args) => {
                Self::run_add_aliases_for_shortcut(&mut client, args).await
            },
            ControlPlaneCommands::Bootstrap(args) => Self::run_bootstrap(&mut client, args).await,
            ControlPlaneCommands::CreateShortcut(args) => Self::run_create_shortcut(&mut client, args).await,
            ControlPlaneCommands::DeleteAliasForShortcut(args) => {
                Self::run_delete_alias_for_shortcut(&mut client, args).await
            },
            ControlPlaneCommands::DeleteShortcut(args) => Self::run_delete_shortcut(&mut client, args).await,
            ControlPlaneCommands::GetDefaultFallbackShortcut => todo!(),
            ControlPlaneCommands::GetShortcut(args) => Self::run_get_shortcut(&mut client, args).await,
            ControlPlaneCommands::ListShortcuts(args) => Self::run_list_shortcuts(&mut client, args).await,
            ControlPlaneCommands::SetDefaultFallbackShortcut(args) => {
                Self::run_set_default_fallback_shortcut(&mut client, args).await
            },
            ControlPlaneCommands::UpdateShortcut(args) => Self::run_update_shortcut(&mut client, args).await,
        }
    }
}
