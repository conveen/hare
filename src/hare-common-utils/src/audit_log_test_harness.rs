//! Test harness for audit record factory implementations.
//!
//! This module provides a trait-based abstraction for testing different
//! `AuditRecordFactory` implementations (e.g., axum-based, tonic-based)
//! with shared test logic.

use std::time::SystemTime;

use crate::audit_log::AuditRecordFactory;

/// Trait for abstracting differences between audit factory implementations in tests.
///
/// Each server implementation (web, control plane) implements this trait to provide
/// the framework-specific request/response building logic, allowing shared test
/// functions to work across all implementations.
pub trait AuditFactoryTestHarness {
    /// The HTTP body type used by this factory.
    type Body: Default;

    /// The factory type being tested.
    type Factory: AuditRecordFactory<RequestBody = Self::Body, ResponseBody = Self::Body> + Default;

    /// Build a request with TCP connection info but without request ID extension.
    fn build_tcp_request(
        uri: &str,
        headers: Option<Vec<(http::header::HeaderName, &str)>>,
    ) -> http::Request<Self::Body>;

    /// Build a request with TCP connection info and required extensions including request ID.
    fn build_tcp_request_with_request_id(
        uri: &str,
        headers: Option<Vec<(http::header::HeaderName, &str)>>,
    ) -> http::Request<Self::Body> {
        let mut request = Self::build_tcp_request(uri, headers);
        request.extensions_mut().insert(tower_http::request_id::RequestId::new(
            crate::request_id::RequestIdGenerator::generate_request_id().get_id().parse().unwrap(),
        ));
        request
    }

    /// Build a request with Unix socket connection info and required extensions.
    #[cfg(unix)]
    fn build_uds_request_with_request_id(
        uri: &str,
        pid: i32,
        uid: u32,
        socket_path: &str,
    ) -> http::Request<Self::Body>;
}

// ============================================================================
// Generic Test Functions
// ============================================================================

/// Test that URL is extracted from request.
pub fn test_extracts_url<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/api/v1/users?page=1", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_request_url, "/api/v1/users?page=1");
}

/// Test that no referer is extracted when header is not present.
pub fn test_no_referer_when_missing<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_request_referer, None);
}

/// Test that referer header is extracted.
pub fn test_extracts_referer<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id(
        "/test",
        Some(vec![(http::header::REFERER, "https://example.com/source")]),
    );

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_request_referer, Some(vec!["https://example.com/source".to_string()]));
}

/// Test that user agent header is extracted.
pub fn test_extracts_user_agent<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id(
        "/test",
        Some(vec![(http::header::USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64)")]),
    );

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_request_user_agent, Some(vec!["Mozilla/5.0 (X11; Linux x86_64)".to_string()]));
}

/// Test that only the last forwarded header is extracted.
pub fn test_extracts_last_forwarded<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let mut request = H::build_tcp_request_with_request_id("/test", None);

    request.headers_mut().append(http::header::FORWARDED, "for=10.0.0.1".parse().unwrap());
    request.headers_mut().append(http::header::FORWARDED, "for=10.0.0.2".parse().unwrap());

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_request_x_forwarded_for, Some(vec!["for=10.0.0.2".to_string()]));
}

/// Test that request ID is extracted.
pub fn test_extracts_request_id<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert!(!audit_record.http_request_uid.is_empty());
}

/// Test that missing request ID causes a panic.
pub fn test_panics_when_request_id_missing<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request("/test", None);

    let start_time = SystemTime::now();
    let _ = factory.extract_from_request(&request, start_time);
}

/// Test that TCP connection info is extracted.
pub fn test_extracts_tcp_connection_info<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert!(audit_record.src_endpoint_ip.is_some());
    assert!(audit_record.src_endpoint_port.is_some());
    assert!(audit_record.dst_endpoint_ip.is_some());
    assert!(audit_record.dst_endpoint_port.is_some());
    assert_eq!(audit_record.dst_endpoint_socket_file_path, None);
    assert_eq!(audit_record.actor_process_pid, None);
    assert_eq!(audit_record.actor_process_user_uid, None);
}

/// Test that long headers are truncated and flagged.
pub fn test_truncates_long_header<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let long_user_agent = "x".repeat(2000);
    let mut request = H::build_tcp_request_with_request_id("/test", None);

    request.headers_mut().insert(http::header::USER_AGENT, long_user_agent.parse().unwrap());

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert!(audit_record.metadata_is_truncated);
}

/// Test that multiple truncated headers set the truncation flag.
pub fn test_truncates_multiple_headers<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let long_value = "x".repeat(2000);
    let mut request = H::build_tcp_request_with_request_id("/test", None);

    request.headers_mut().insert(http::header::USER_AGENT, long_value.clone().parse().unwrap());
    request.headers_mut().insert(http::header::REFERER, long_value.parse().unwrap());

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert!(audit_record.metadata_is_truncated);
}

/// Test that 200 response code is extracted correctly.
pub fn test_extracts_response_code_200<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_response_code, 200);
    assert_eq!(audit_record.status_id, 1);
    assert_eq!(audit_record.status_code, None);
}

/// Test that 404 response code is extracted correctly.
pub fn test_extracts_response_code_404<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(404).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_response_code, 404);
    assert_eq!(audit_record.status_id, 2);
    assert_eq!(audit_record.status_code, Some("NotFound".to_string()));
}

/// Test that 500 response code is extracted correctly.
pub fn test_extracts_response_code_500<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(500).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_response_code, 500);
    assert_eq!(audit_record.status_id, 2);
    assert_eq!(audit_record.status_code, Some("InternalError".to_string()));
}

/// Test that content length is extracted from response.
pub fn test_extracts_content_length<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response =
        http::Response::builder().status(200).header("Content-Length", "1024").body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_response_length, 1024);
}

/// Test that missing content length defaults to zero.
pub fn test_missing_content_length_defaults_to_zero<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.http_response_length, 0);
}

/// Test that timestamps are in ISO-8601 format.
pub fn test_timestamps_are_iso8601<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let fixed_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1577836800);
    let builder = factory.extract_from_request(&request, fixed_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.start_time, "2020-01-01T00:00:00.000Z");
    assert!(!audit_record.end_time.is_empty());
    assert!(audit_record.end_time.ends_with("Z"));
}

/// Test that end time is after start time.
pub fn test_end_time_after_start_time<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_tcp_request_with_request_id("/test", None);

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    std::thread::sleep(std::time::Duration::from_millis(10));

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert!(!audit_record.start_time.is_empty());
    assert!(!audit_record.end_time.is_empty());
    assert!(audit_record.end_time.as_str() >= audit_record.start_time.as_str());
}

/// Test that Unix socket connection info is extracted.
#[cfg(unix)]
pub fn test_extracts_unix_socket_info<H: AuditFactoryTestHarness>() {
    let factory = H::Factory::default();
    let request = H::build_uds_request_with_request_id("/test", 1234, 1000, "/tmp/hare.sock");

    let start_time = SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);

    let response = http::Response::builder().status(200).body(H::Body::default()).unwrap();
    let audit_record = factory.extract_from_response(&response, builder);

    assert_eq!(audit_record.actor_process_pid, Some(1234));
    assert_eq!(audit_record.actor_process_user_uid, Some(1000));
    assert_eq!(audit_record.src_endpoint_ip, None);
    assert_eq!(audit_record.src_endpoint_port, None);
    assert_eq!(audit_record.dst_endpoint_ip, None);
    assert_eq!(audit_record.dst_endpoint_port, None);
    assert_eq!(audit_record.dst_endpoint_socket_file_path, Some("/tmp/hare.sock".to_string()));
}

/// Macro to generate test functions for an audit factory implementation.
///
/// # Example
///
/// ```ignore
/// use hare_common_utils::audit_factory_tests;
///
/// struct MyHarness;
/// impl AuditFactoryTestHarness for MyHarness { /* ... */ }
///
/// audit_factory_tests!(MyHarness);
/// ```
#[macro_export]
macro_rules! audit_factory_tests {
    ($harness:ty) => {
        #[test]
        fn test_when_request_with_url_then_extracts_url() {
            $crate::audit_log_test_harness::test_extracts_url::<$harness>();
        }

        #[test]
        fn test_when_request_without_referer_then_no_referer_extracted() {
            $crate::audit_log_test_harness::test_no_referer_when_missing::<$harness>();
        }

        #[test]
        fn test_when_request_with_referer_then_extracts_referer() {
            $crate::audit_log_test_harness::test_extracts_referer::<$harness>();
        }

        #[test]
        fn test_when_request_with_user_agent_then_extracts_user_agent() {
            $crate::audit_log_test_harness::test_extracts_user_agent::<$harness>();
        }

        #[test]
        fn test_when_request_with_forwarded_header_then_extracts_last_forwarded() {
            $crate::audit_log_test_harness::test_extracts_last_forwarded::<$harness>();
        }

        #[test]
        fn test_when_request_with_request_id_then_extracts_request_id() {
            $crate::audit_log_test_harness::test_extracts_request_id::<$harness>();
        }

        #[test]
        #[should_panic(expected = "RequestId must be set")]
        fn test_when_request_without_request_id_then_panics() {
            $crate::audit_log_test_harness::test_panics_when_request_id_missing::<$harness>();
        }

        #[test]
        fn test_when_request_with_tcp_connection_then_extracts_connection_info() {
            $crate::audit_log_test_harness::test_extracts_tcp_connection_info::<$harness>();
        }

        #[test]
        fn test_when_request_with_long_header_then_truncates_and_flags() {
            $crate::audit_log_test_harness::test_truncates_long_header::<$harness>();
        }

        #[test]
        fn test_when_request_with_multiple_truncated_headers_then_metadata_is_truncated() {
            $crate::audit_log_test_harness::test_truncates_multiple_headers::<$harness>();
        }

        #[test]
        fn test_when_response_status_200_then_extracts_code() {
            $crate::audit_log_test_harness::test_extracts_response_code_200::<$harness>();
        }

        #[test]
        fn test_when_response_status_404_then_extracts_code_and_status() {
            $crate::audit_log_test_harness::test_extracts_response_code_404::<$harness>();
        }

        #[test]
        fn test_when_response_status_500_then_extracts_code_and_status() {
            $crate::audit_log_test_harness::test_extracts_response_code_500::<$harness>();
        }

        #[test]
        fn test_when_response_with_content_length_then_extracts_length() {
            $crate::audit_log_test_harness::test_extracts_content_length::<$harness>();
        }

        #[test]
        fn test_when_response_without_content_length_then_defaults_to_zero() {
            $crate::audit_log_test_harness::test_missing_content_length_defaults_to_zero::<$harness>();
        }

        #[test]
        fn test_when_fixed_start_time_then_timestamps_are_iso8601_format() {
            $crate::audit_log_test_harness::test_timestamps_are_iso8601::<$harness>();
        }

        #[test]
        fn test_when_request_then_response_then_start_time_before_end_time() {
            $crate::audit_log_test_harness::test_end_time_after_start_time::<$harness>();
        }

        #[cfg(unix)]
        #[test]
        fn test_when_request_with_unix_socket_then_extracts_socket_info() {
            $crate::audit_log_test_harness::test_extracts_unix_socket_info::<$harness>();
        }
    };
}
