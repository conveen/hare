use valuable::{Fields, NamedField, NamedValues, StructDef, Structable, Valuable, Value, Visit};

// Static field definitions for Valuable implementation
static FIELD_ACTOR_PROCESS_PID: NamedField<'static> = NamedField::new("actor_process_pid");
static FIELD_ACTOR_PROCESS_USER_UID: NamedField<'static> = NamedField::new("actor_process_user_uid");
static FIELD_ACTOR_USER_ACCOUNT_UID: NamedField<'static> = NamedField::new("actor_user_account_uid");
static FIELD_ACTOR_USER_NAME: NamedField<'static> = NamedField::new("actor_user_name");
static FIELD_ACTOR_USER_UID: NamedField<'static> = NamedField::new("actor_user_uid");
static FIELD_DST_ENDPOINT_IP: NamedField<'static> = NamedField::new("dst_endpoint_ip");
static FIELD_DST_ENDPOINT_PORT: NamedField<'static> = NamedField::new("dst_endpoint_port");
static FIELD_DST_ENDPOINT_SOCKET_FILE_PATH: NamedField<'static> = NamedField::new("dst_endpoint_socket_file_path");
static FIELD_END_TIME: NamedField<'static> = NamedField::new("end_time");
static FIELD_HTTP_REQUEST_REFERER: NamedField<'static> = NamedField::new("http_request_referer");
static FIELD_HTTP_REQUEST_UID: NamedField<'static> = NamedField::new("http_request_uid");
static FIELD_HTTP_REQUEST_URL: NamedField<'static> = NamedField::new("http_request_url");
static FIELD_HTTP_REQUEST_USER_AGENT: NamedField<'static> = NamedField::new("http_request_user_agent");
static FIELD_HTTP_REQUEST_X_FORWARDED_FOR: NamedField<'static> = NamedField::new("http_request_x_forwarded_for");
static FIELD_HTTP_RESPONSE_CODE: NamedField<'static> = NamedField::new("http_response_code");
static FIELD_HTTP_RESPONSE_LENGTH: NamedField<'static> = NamedField::new("http_response_length");
static FIELD_METADATA_IS_TRUNCATED: NamedField<'static> = NamedField::new("metadata_is_truncated");
static FIELD_SRC_ENDPOINT_IP: NamedField<'static> = NamedField::new("src_endpoint_ip");
static FIELD_SRC_ENDPOINT_PORT: NamedField<'static> = NamedField::new("src_endpoint_port");
static FIELD_START_TIME: NamedField<'static> = NamedField::new("start_time");
static FIELD_STATUS_ID: NamedField<'static> = NamedField::new("status_id");
static FIELD_STATUS_CODE: NamedField<'static> = NamedField::new("status_code");
static FIELD_STATUS_DETAILS: NamedField<'static> = NamedField::new("status_details");
static FIELD_TIME: NamedField<'static> = NamedField::new("time");
static FIELD_TLS_CIPHER: NamedField<'static> = NamedField::new("tls_cipher");
static FIELD_TLS_VERSION: NamedField<'static> = NamedField::new("tls_version");

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
#[derive(bon::Builder, Debug)]
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

impl Valuable for AuditRecord {
    fn as_value(&self) -> Value<'_> {
        Value::Structable(self)
    }

    fn visit(&self, visitor: &mut dyn Visit) {
        // Only emit populated optional fields
        if let Some(ref pid) = self.actor_process_pid {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_ACTOR_PROCESS_PID], &[pid.as_value()]));
        }
        if let Some(ref uid) = self.actor_process_user_uid {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_ACTOR_PROCESS_USER_UID], &[uid.as_value()]));
        }
        if let Some(ref uid) = self.actor_user_account_uid {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_ACTOR_USER_ACCOUNT_UID], &[uid.as_value()]));
        }
        if let Some(ref name) = self.actor_user_name {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_ACTOR_USER_NAME], &[name.as_value()]));
        }
        if let Some(ref uid) = self.actor_user_uid {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_ACTOR_USER_UID], &[uid.as_value()]));
        }
        if let Some(ref ip) = self.dst_endpoint_ip {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_DST_ENDPOINT_IP], &[ip.as_value()]));
        }
        if let Some(ref port) = self.dst_endpoint_port {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_DST_ENDPOINT_PORT], &[port.as_value()]));
        }
        if let Some(ref path) = self.dst_endpoint_socket_file_path {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_DST_ENDPOINT_SOCKET_FILE_PATH], &[path.as_value()]));
        }

        // Always emit required fields
        visitor.visit_named_fields(&NamedValues::new(&[FIELD_END_TIME], &[self.end_time.as_value()]));

        if let Some(ref referer) = self.http_request_referer {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_HTTP_REQUEST_REFERER], &[referer.as_value()]));
        }
        visitor.visit_named_fields(&NamedValues::new(&[FIELD_HTTP_REQUEST_UID], &[self.http_request_uid.as_value()]));
        visitor.visit_named_fields(&NamedValues::new(&[FIELD_HTTP_REQUEST_URL], &[self.http_request_url.as_value()]));

        if let Some(ref ua) = self.http_request_user_agent {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_HTTP_REQUEST_USER_AGENT], &[ua.as_value()]));
        }
        if let Some(ref xff) = self.http_request_x_forwarded_for {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_HTTP_REQUEST_X_FORWARDED_FOR], &[xff.as_value()]));
        }

        visitor
            .visit_named_fields(&NamedValues::new(&[FIELD_HTTP_RESPONSE_CODE], &[self.http_response_code.as_value()]));
        visitor.visit_named_fields(&NamedValues::new(
            &[FIELD_HTTP_RESPONSE_LENGTH],
            &[self.http_response_length.as_value()],
        ));
        visitor.visit_named_fields(&NamedValues::new(
            &[FIELD_METADATA_IS_TRUNCATED],
            &[self.metadata_is_truncated.as_value()],
        ));

        if let Some(ref ip) = self.src_endpoint_ip {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_SRC_ENDPOINT_IP], &[ip.as_value()]));
        }
        if let Some(ref port) = self.src_endpoint_port {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_SRC_ENDPOINT_PORT], &[port.as_value()]));
        }

        visitor.visit_named_fields(&NamedValues::new(&[FIELD_START_TIME], &[self.start_time.as_value()]));
        visitor.visit_named_fields(&NamedValues::new(&[FIELD_STATUS_ID], &[self.status_id.as_value()]));

        if let Some(ref code) = self.status_code {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_STATUS_CODE], &[code.as_value()]));
        }
        if let Some(ref details) = self.status_details {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_STATUS_DETAILS], &[details.as_value()]));
        }

        visitor.visit_named_fields(&NamedValues::new(&[FIELD_TIME], &[self.time.as_value()]));

        if let Some(ref cipher) = self.tls_cipher {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_TLS_CIPHER], &[cipher.as_value()]));
        }
        if let Some(ref version) = self.tls_version {
            visitor.visit_named_fields(&NamedValues::new(&[FIELD_TLS_VERSION], &[version.as_value()]));
        }
    }
}

impl Structable for AuditRecord {
    fn definition(&self) -> StructDef<'_> {
        // Use dynamic definition since we filter fields at runtime
        StructDef::new_dynamic("AuditRecord", Fields::Named(&[]))
    }
}
