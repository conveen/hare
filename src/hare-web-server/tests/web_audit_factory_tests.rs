use hare_common_utils::{
    audit_factory_tests, audit_log_test_harness::AuditFactoryTestHarness, connect_info::ConnectionInformation,
};
use hare_web_server::audit_log::WebAuditRecordFactory;

/// Test harness for WebAuditRecordFactory.
struct WebAuditFactoryTestHarness;

impl AuditFactoryTestHarness for WebAuditFactoryTestHarness {
    type Body = axum::body::Body;
    type Factory = WebAuditRecordFactory;

    fn build_tcp_request(
        uri: &str,
        headers: Option<Vec<(http::header::HeaderName, &str)>>,
    ) -> http::Request<Self::Body> {
        let mut builder = http::Request::builder().method("GET").uri(uri);

        if let Some(headers) = headers {
            for (name, value) in headers {
                builder = builder.header(name, value);
            }
        }

        let mut request = builder.body(axum::body::Body::empty()).unwrap();

        request.extensions_mut().insert(axum::extract::ConnectInfo(ConnectionInformation {
            actor_process_pid: None,
            actor_process_user_uid: None,
            src_endpoint_ip: Some("192.168.1.100".to_string()),
            src_endpoint_port: Some(54321),
            dst_endpoint_ip: Some("127.0.0.1".to_string()),
            dst_endpoint_port: Some(8080),
            dst_endpoint_socket_file_path: None,
        }));

        request
    }

    #[cfg(unix)]
    fn build_uds_request_with_request_id(
        uri: &str,
        pid: i32,
        uid: u32,
        socket_path: &str,
    ) -> http::Request<Self::Body> {
        let mut request = http::Request::builder().method("GET").uri(uri).body(axum::body::Body::empty()).unwrap();

        request.extensions_mut().insert(axum::extract::ConnectInfo(ConnectionInformation {
            actor_process_pid: Some(pid),
            actor_process_user_uid: Some(uid),
            src_endpoint_ip: None,
            src_endpoint_port: None,
            dst_endpoint_ip: None,
            dst_endpoint_port: None,
            dst_endpoint_socket_file_path: Some(socket_path.to_string()),
        }));

        request.extensions_mut().insert(tower_http::request_id::RequestId::new(
            hare_common_utils::request_id::RequestIdGenerator::generate_request_id().get_id().parse().unwrap(),
        ));

        request
    }
}

audit_factory_tests!(WebAuditFactoryTestHarness);
