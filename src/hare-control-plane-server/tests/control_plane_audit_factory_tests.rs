use std::net::SocketAddr;

use hare_common_utils::{audit_factory_tests, audit_log_test_harness::AuditFactoryTestHarness};
use hare_control_plane_server::audit_log::ControlPlaneAuditRecordFactory;

/// Test harness for ControlPlaneAuditRecordFactory.
struct ControlPlaneAuditFactoryTestHarness;

impl AuditFactoryTestHarness for ControlPlaneAuditFactoryTestHarness {
    type Body = tonic::body::Body;
    type Factory = ControlPlaneAuditRecordFactory;

    fn build_tcp_request(
        uri: &str,
        headers: Option<Vec<(http::header::HeaderName, &str)>>,
    ) -> http::Request<Self::Body> {
        let mut builder = http::Request::builder().method("POST").uri(uri);

        if let Some(headers) = headers {
            for (name, value) in headers {
                builder = builder.header(name, value);
            }
        }

        let mut request = builder.body(tonic::body::Body::default()).unwrap();

        let local_addr: SocketAddr = "127.0.0.1:50051".parse().unwrap();
        let remote_addr: SocketAddr = "192.168.1.100:54321".parse().unwrap();
        let tcp_connect_info =
            tonic::transport::server::TcpConnectInfo { local_addr: Some(local_addr), remote_addr: Some(remote_addr) };
        request.extensions_mut().insert(tcp_connect_info);

        request
    }

    #[cfg(unix)]
    fn build_uds_request_with_request_id(
        uri: &str,
        pid: i32,
        uid: u32,
        socket_path: &str,
    ) -> http::Request<Self::Body> {
        use std::sync::Arc;
        use tokio::net::unix::UCred;

        let mut request = http::Request::builder().method("POST").uri(uri).body(tonic::body::Body::default()).unwrap();

        // SAFETY: tokio::net::unix::SocketAddr wraps std::os::unix::net::SocketAddr with the same layout.
        let std_addr = std::os::unix::net::SocketAddr::from_pathname(socket_path).unwrap();
        let peer_addr: tokio::net::unix::SocketAddr = unsafe { std::mem::transmute(std_addr) };

        // SAFETY: UCred doesn't have a public constructor, so we use transmute to create one for testing.
        // This assumes the internal layout matches (pid: Option<i32>, uid: u32, gid: u32).
        let peer_cred: UCred = unsafe { std::mem::transmute((Some(pid), uid, uid)) };

        let uds_connect_info = tonic::transport::server::UdsConnectInfo {
            peer_addr: Some(Arc::new(peer_addr)),
            peer_cred: Some(peer_cred),
        };
        request.extensions_mut().insert(uds_connect_info);

        request.extensions_mut().insert(tower_http::request_id::RequestId::new(
            hare_common_utils::request_id::RequestIdGenerator::generate_request_id().get_id().parse().unwrap(),
        ));

        request
    }
}

audit_factory_tests!(ControlPlaneAuditFactoryTestHarness);
