use hare_common_utils::{
    audit_log,
    audit_record::{
        audit_record_builder::{
            SetActorProcessPid, SetActorProcessUserUid, SetDstEndpointIp, SetDstEndpointPort,
            SetDstEndpointSocketFilePath, SetHttpRequestReferer, SetHttpRequestUid, SetHttpRequestUrl,
            SetHttpRequestUserAgent, SetHttpRequestXForwardedFor, SetMetadataIsTruncated, SetSrcEndpointIp,
            SetSrcEndpointPort, SetStartTime, SetTime,
        },
        AuditRecord,
    },
    connect_info,
};

/// Factory for creating audit records from Axum HTTP requests and responses.
#[derive(Debug, Clone, Default)]
pub struct WebAuditRecordFactory;

impl audit_log::AuditRecordFactory for WebAuditRecordFactory {
    type RequestBody = axum::body::Body;
    type ResponseBody = axum::body::Body;
    type RequestExtractorRecordBuilderState = SetTime<
        SetStartTime<
            SetSrcEndpointPort<
                SetSrcEndpointIp<
                    SetMetadataIsTruncated<
                        SetHttpRequestUid<
                            SetHttpRequestXForwardedFor<
                                SetHttpRequestUserAgent<
                                    SetHttpRequestReferer<
                                        SetHttpRequestUrl<
                                            SetDstEndpointSocketFilePath<
                                                SetDstEndpointPort<
                                                    SetDstEndpointIp<SetActorProcessUserUid<SetActorProcessPid>>,
                                                >,
                                            >,
                                        >,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >;

    fn extract_from_request(
        &self,
        request: &http::Request<Self::RequestBody>,
        start_time: std::time::SystemTime,
    ) -> hare_common_utils::audit_record::AuditRecordBuilder<Self::RequestExtractorRecordBuilderState> {
        // Extract request headers with truncation tracking
        let request_headers = request.headers();
        let (http_request_referer, referer_truncated) =
            audit_log::extract_header_values(request_headers, http::header::REFERER, None, None);
        let (http_request_user_agent, ua_truncated) =
            audit_log::extract_header_values(request_headers, http::header::USER_AGENT, None, None);
        let (http_request_x_forwarded_for, forwarded_truncated) =
            // Only extract the last XFF header
            // Avoids capturing arbitrary header values set by untrusted proxies earlier in the network chain
            // Assumes the last proxy is trusted to set the XFF, if not using a proxy change to HeaderPosition::All
            audit_log::extract_header_values(request_headers, http::header::FORWARDED, None, Some(audit_log::HeaderPosition::Last));

        // Extract request ID from extensions (set by SetRequestIdLayer)
        let http_request_uid = request
            .extensions()
            .get::<tower_http::request_id::RequestId>()
            .expect("RequestId must be set")
            .header_value()
            .to_str()
            .expect("RequestId must be valid UTF-8")
            .to_string();

        // Extract client and server connection information
        let connect_info = request
            .extensions()
            .get::<axum::extract::ConnectInfo<connect_info::ConnectionInformation>>()
            .expect("ConnectInfo extension must be present");

        // Calculate request start timestamp
        let timestamp = chrono::DateTime::<chrono::Utc>::from(start_time).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        AuditRecord::builder()
            .maybe_actor_process_pid(connect_info.actor_process_pid)
            .maybe_actor_process_user_uid(connect_info.actor_process_user_uid)
            .maybe_dst_endpoint_ip(connect_info.dst_endpoint_ip.clone())
            .maybe_dst_endpoint_port(connect_info.dst_endpoint_port)
            .maybe_dst_endpoint_socket_file_path(connect_info.dst_endpoint_socket_file_path.clone())
            .http_request_url(request.uri().to_string())
            .maybe_http_request_referer(http_request_referer)
            .maybe_http_request_user_agent(http_request_user_agent)
            .maybe_http_request_x_forwarded_for(http_request_x_forwarded_for)
            .http_request_uid(http_request_uid)
            .metadata_is_truncated(referer_truncated || ua_truncated || forwarded_truncated)
            .maybe_src_endpoint_ip(connect_info.src_endpoint_ip.clone())
            .maybe_src_endpoint_port(connect_info.src_endpoint_port)
            .start_time(timestamp.clone())
            .time(timestamp)
    }

    fn extract_from_response(
        &self,
        response: &http::Response<Self::ResponseBody>,
        builder: hare_common_utils::audit_record::AuditRecordBuilder<Self::RequestExtractorRecordBuilderState>,
    ) -> AuditRecord {
        // Calculate end_time as the current time
        let end_time = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        builder
            .http_response_code(hare_common_utils::audit_log::extract_http_response_code(response))
            .http_response_length(hare_common_utils::audit_log::extract_http_response_length(response.headers()))
            .status_id(hare_common_utils::audit_log::extract_status_id(response))
            .maybe_status_code(hare_common_utils::audit_log::extract_status_code(response))
            .end_time(end_time)
            .build()
    }
}
