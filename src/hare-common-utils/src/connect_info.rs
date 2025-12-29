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

#[cfg(unix)]
impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_, tokio::net::UnixListener>>
    for ConnectionInformation
{
    fn connect_info(stream: axum::serve::IncomingStream<'_, tokio::net::UnixListener>) -> Self {
        Self {
            actor_process_pid: stream.io().peer_cred().ok().map(|cred| cred.pid()).flatten(),
            actor_process_user_uid: stream.io().peer_cred().ok().map(|cred| cred.uid()),
            src_endpoint_ip: None,
            src_endpoint_port: None,
            dst_endpoint_ip: None,
            dst_endpoint_port: None,
            dst_endpoint_socket_file_path: stream
                .io()
                .local_addr()
                .ok()
                .map(|addr| addr.as_pathname().map(|path| path.to_str().unwrap().to_string()))
                .flatten(),
        }
    }
}
