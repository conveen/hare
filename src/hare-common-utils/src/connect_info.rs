/// Connection information for incoming connections.
///
/// This was previously defined in `audit_log.rs` but has been moved here
/// to separate connection-centric types from audit middleware logic.
#[derive(Clone, Debug)]
pub struct ConnectionInformation {
    pub actor_process_pid: Option<i32>,
    pub actor_process_user_uid: Option<u32>,
    pub dst_endpoint_ip: Option<String>,
    pub dst_endpoint_port: Option<u16>,
    pub dst_endpoint_socket_file_path: Option<String>,
    pub src_endpoint_ip: Option<String>,
    pub src_endpoint_port: Option<u16>,
}

#[cfg(feature = "axum")]
impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_, tokio::net::TcpListener>>
    for ConnectionInformation
{
    fn connect_info(stream: axum::serve::IncomingStream<'_, tokio::net::TcpListener>) -> Self {
        Self {
            actor_process_pid: None,
            actor_process_user_uid: None,
            src_endpoint_ip: Some(stream.remote_addr().ip().to_string()),
            src_endpoint_port: Some(stream.remote_addr().port()),
            dst_endpoint_ip: stream.io().local_addr().ok().map(|addr| addr.ip().to_string()),
            dst_endpoint_port: stream.io().local_addr().ok().map(|addr| addr.port()),
            dst_endpoint_socket_file_path: None,
        }
    }
}

#[cfg(all(unix, feature = "axum"))]
impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_, tokio::net::UnixListener>>
    for ConnectionInformation
{
    fn connect_info(stream: axum::serve::IncomingStream<'_, tokio::net::UnixListener>) -> Self {
        Self {
            actor_process_pid: stream.io().peer_cred().ok().and_then(|cred| cred.pid()),
            actor_process_user_uid: stream.io().peer_cred().ok().map(|cred| cred.uid()),
            src_endpoint_ip: None,
            src_endpoint_port: None,
            dst_endpoint_ip: None,
            dst_endpoint_port: None,
            dst_endpoint_socket_file_path: stream
                .io()
                .local_addr()
                .ok()
                .and_then(|addr| addr.as_pathname().map(|path| path.to_str().unwrap().to_string())),
        }
    }
}

#[cfg(feature = "tonic")]
impl From<&tonic::transport::server::TcpConnectInfo> for ConnectionInformation {
    fn from(info: &tonic::transport::server::TcpConnectInfo) -> Self {
        Self {
            actor_process_pid: None,
            actor_process_user_uid: None,
            src_endpoint_ip: info.remote_addr().map(|a| a.ip().to_string()),
            src_endpoint_port: info.remote_addr().map(|a| a.port()),
            dst_endpoint_ip: info.local_addr().map(|a| a.ip().to_string()),
            dst_endpoint_port: info.local_addr().map(|a| a.port()),
            dst_endpoint_socket_file_path: None,
        }
    }
}

#[cfg(all(unix, feature = "tonic"))]
impl From<&tonic::transport::server::UdsConnectInfo> for ConnectionInformation {
    fn from(info: &tonic::transport::server::UdsConnectInfo) -> Self {
        Self {
            actor_process_pid: info.peer_cred.as_ref().and_then(|c| c.pid()),
            actor_process_user_uid: info.peer_cred.as_ref().map(|c| c.uid()),
            src_endpoint_ip: None,
            src_endpoint_port: None,
            dst_endpoint_ip: None,
            dst_endpoint_port: None,
            dst_endpoint_socket_file_path: info
                .peer_addr
                .as_ref()
                .and_then(|addr| addr.as_pathname().map(|path| path.to_str().unwrap().to_string())),
        }
    }
}
