/// An audit record of a single request-response from the service.
///
/// A good audit record captures:
///     * When (timestamp of the event)
///     * Who (the actor that performed an action or triggered the event)
///     * What (the resources impacted by the request)
///     * Result (whether the action was successful and the resulting impact)
///
/// The fields in this record are modelled after the Open Cybersecurity Schema Framework
/// [Network Activity](https://schema.ocsf.io/1.6.0/classes/network_activity),
/// an open source normalization framework housed under the Linux Foundation.
#[derive(bon::Builder, Debug, valuable::Valuable)]
#[builder(derive(Debug, Clone), state_mod(vis = "pub"))]
pub struct AuditRecord {
    /// The PID of the process that initiated the connection.
    /// For Unix domain socket connections only.
    pub actor_process_pid: Option<i32>,
    /// The UID of the user that owned the connection-initiating process.
    /// For Unix domain socket connections only.
    pub actor_process_user_uid: Option<u32>,
    /// Account identifier of the calling identity.
    pub actor_user_account_uid: Option<String>,
    /// Name of the calling identity.
    pub actor_user_name: Option<String>,
    /// Identifier of the calling identity.
    pub actor_user_uid: Option<String>,
    /// IP address of the destination endpoint (node that received the request).
    /// None for Unix domain sockets which do not use TCP/IP.
    pub dst_endpoint_ip: Option<String>,
    /// Port of the destination endpoint (node that received the request).
    /// None for Unix domain sockets which do not use TCP/IP.
    pub dst_endpoint_port: Option<u16>,
    /// The full path to the connection socket file.
    /// For Unix domain socket connections only.
    pub dst_endpoint_socket_file_path: Option<String>,
    /// Timestamp when request processing finished.
    pub end_time: String,
    /// HTTP [Referer header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Referer).
    pub http_request_referer: Option<Vec<String>>,
    /// Identifier of the request.
    pub http_request_uid: String,
    /// URL of the request.
    pub http_request_url: String,
    /// HTTP [User Agent header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/User-Agent).
    pub http_request_user_agent: Option<Vec<String>>,
    /// HTTP [X-Forwarded-For header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/X-Forwarded-For).
    pub http_request_x_forwarded_for: Option<Vec<String>>,
    /// HTTP [response code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status).
    pub http_response_code: u16,
    /// Respone size in bytes.
    ///
    /// This is the total response size including headers.
    pub http_response_length: u64,
    /// Whether any or all fields in the audit record are truncated.
    pub metadata_is_truncated: bool,
    /// IP address of the source endpoint.
    ///
    /// Client IP address (prior network hop, could be a proxy/load balancer).
    /// The original client IP address can be determined with this and XFF together,
    /// though XFF cannot always be trusted through the whole network chain.
    /// None for Unix domain sockets which do not use TCP/IP.
    pub src_endpoint_ip: Option<String>,
    /// Port of the source endpoint.
    ///
    /// Client port (prior network hop, could be a proxy/load balancer).
    /// None for Unix domain sockets which do not use TCP/IP.
    pub src_endpoint_port: Option<u16>,
    /// Timestamp when request was received.
    pub start_time: String,
    /// Status of the request.
    ///
    /// 0: Unknown
    /// 1: Success
    /// 2: Failure
    /// 99: Other
    pub status_id: u16,
    /// Status code of the request (e.g. error code)
    pub status_code: Option<String>,
    /// Additional information about the request status (e.g. error message).
    pub status_details: Option<String>,
    /// Timestamp when request was received.
    pub time: String,
    /// Cipher of the negotatied TLS session.
    pub tls_cipher: Option<String>,
    /// Version of the negotatied TLS session.
    pub tls_version: Option<String>,
}
