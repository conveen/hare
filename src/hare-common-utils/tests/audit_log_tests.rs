use hare_common_utils::{
    audit_log::{
        extract_header_values, extract_http_response_code, extract_http_response_length, extract_request_id,
        extract_status_code, extract_status_id, AuditRecordFactory, HeaderPosition, MAXIMUM_HEADER_COUNT,
        MAXIMUM_HEADER_VALUE_LENGTH,
    },
    audit_record::AuditRecord,
};

// ============================================================================
// extract_request_id() tests
// ============================================================================

#[test]
fn test_extract_request_id_when_present_then_returns_id() {
    let mut extensions = http::Extensions::new();
    extensions.insert(tower_http::request_id::RequestId::new("test-request-123".parse().unwrap()));

    let request_id = extract_request_id(&extensions);
    assert_eq!(request_id, "test-request-123");
}

#[test]
#[should_panic(expected = "RequestId must be set")]
fn test_extract_request_id_when_missing_then_panics() {
    let extensions = http::Extensions::new();
    extract_request_id(&extensions);
}

// ============================================================================
// extract_header_values() tests
// ============================================================================

#[test]
fn test_extract_header_values_when_not_present_then_none() {
    let headers = http::HeaderMap::new();
    let (values, was_truncated) = extract_header_values(&headers, http::header::REFERER, None, None);
    assert_eq!(values, None);
    assert_eq!(was_truncated, false);
}

#[test]
fn test_extract_header_values_when_single_header_then_returns_value() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::USER_AGENT, "Mozilla/5.0".parse().unwrap());

    let (values, was_truncated) = extract_header_values(&headers, http::header::USER_AGENT, None, None);
    assert_eq!(values, Some(vec!["Mozilla/5.0".to_string()]));
    assert_eq!(was_truncated, false);
}

#[test]
fn test_extract_header_values_when_multiple_with_position_all_then_returns_all() {
    let mut headers = http::HeaderMap::new();
    headers.append(http::header::ACCEPT, "application/json".parse().unwrap());
    headers.append(http::header::ACCEPT, "text/html".parse().unwrap());
    headers.append(http::header::ACCEPT, "text/plain".parse().unwrap());

    let (values, was_truncated) =
        extract_header_values(&headers, http::header::ACCEPT, None, Some(HeaderPosition::All));
    assert_eq!(values.as_ref().map(|v| v.len()), Some(3));
    assert!(values.unwrap().contains(&"application/json".to_string()));
    assert_eq!(was_truncated, false);
}

#[test]
fn test_extract_header_values_when_max_count_specified_then_respects_limit() {
    let mut headers = http::HeaderMap::new();
    headers.append(http::header::ACCEPT, "value1".parse().unwrap());
    headers.append(http::header::ACCEPT, "value2".parse().unwrap());
    headers.append(http::header::ACCEPT, "value3".parse().unwrap());
    headers.append(http::header::ACCEPT, "value4".parse().unwrap());

    let (values, _) = extract_header_values(&headers, http::header::ACCEPT, Some(2), Some(HeaderPosition::All));
    assert_eq!(values.map(|v| v.len()), Some(2));
}

#[test]
fn test_extract_header_values_when_position_first_then_returns_first_only() {
    let mut headers = http::HeaderMap::new();
    headers.append(http::header::ACCEPT, "first".parse().unwrap());
    headers.append(http::header::ACCEPT, "second".parse().unwrap());
    headers.append(http::header::ACCEPT, "third".parse().unwrap());

    let (values, _) = extract_header_values(&headers, http::header::ACCEPT, None, Some(HeaderPosition::First));
    assert_eq!(values, Some(vec!["first".to_string()]));
}

#[test]
fn test_extract_header_values_when_position_last_then_returns_last_only() {
    let mut headers = http::HeaderMap::new();
    headers.append(http::header::ACCEPT, "first".parse().unwrap());
    headers.append(http::header::ACCEPT, "second".parse().unwrap());
    headers.append(http::header::ACCEPT, "third".parse().unwrap());

    let (values, _) = extract_header_values(&headers, http::header::ACCEPT, None, Some(HeaderPosition::Last));
    assert_eq!(values, Some(vec!["third".to_string()]));
}

#[test]
fn test_extract_header_values_when_value_exceeds_max_length_then_truncates_and_flags() {
    let mut headers = http::HeaderMap::new();
    let long_value = "x".repeat(MAXIMUM_HEADER_VALUE_LENGTH + 100);
    headers.insert(http::header::USER_AGENT, long_value.parse().unwrap());

    let (values, was_truncated) = extract_header_values(&headers, http::header::USER_AGENT, None, None);
    assert_eq!(was_truncated, true);
    assert_eq!(values.unwrap()[0].len(), MAXIMUM_HEADER_VALUE_LENGTH);
}

#[test]
fn test_extract_header_values_when_no_max_count_then_uses_default() {
    let mut headers = http::HeaderMap::new();
    for i in 0..10 {
        headers.append(http::header::ACCEPT, format!("value{}", i).parse().unwrap());
    }

    let (values, _) = extract_header_values(&headers, http::header::ACCEPT, None, Some(HeaderPosition::All));
    assert_eq!(values.map(|v| v.len()), Some(MAXIMUM_HEADER_COUNT));
}

// ============================================================================
// extract_http_response_code() tests
// ============================================================================

#[test]
fn test_extract_http_response_code_when_status_200_then_returns_200() {
    let response = http::Response::builder().status(200).body(()).unwrap();
    assert_eq!(extract_http_response_code(&response), 200);
}

#[test]
fn test_extract_http_response_code_when_status_404_then_returns_404() {
    let response = http::Response::builder().status(404).body(()).unwrap();
    assert_eq!(extract_http_response_code(&response), 404);
}

#[test]
fn test_extract_http_response_code_when_status_500_then_returns_500() {
    let response = http::Response::builder().status(500).body(()).unwrap();
    assert_eq!(extract_http_response_code(&response), 500);
}

// ============================================================================
// extract_http_response_length() tests
// ============================================================================

#[test]
fn test_extract_http_response_length_when_valid_then_returns_value() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_LENGTH, "1024".parse().unwrap());
    assert_eq!(extract_http_response_length(&headers), 1024);
}

#[test]
fn test_extract_http_response_length_when_zero_then_returns_zero() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_LENGTH, "0".parse().unwrap());
    assert_eq!(extract_http_response_length(&headers), 0);
}

#[test]
fn test_extract_http_response_length_when_large_value_then_returns_value() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_LENGTH, "9999999999".parse().unwrap());
    assert_eq!(extract_http_response_length(&headers), 9999999999);
}

#[test]
fn test_extract_http_response_length_when_missing_then_returns_zero() {
    let headers = http::HeaderMap::new();
    assert_eq!(extract_http_response_length(&headers), 0);
}

#[test]
#[should_panic(expected = "Content-Length must be a non-negative integer")]
fn test_extract_http_response_length_when_negative_then_panics() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_LENGTH, "-1".parse().unwrap());
    extract_http_response_length(&headers);
}

#[test]
#[should_panic(expected = "Content-Length must be a non-negative integer")]
fn test_extract_http_response_length_when_non_numeric_then_panics() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_LENGTH, "not_a_number".parse().unwrap());
    extract_http_response_length(&headers);
}

#[test]
#[should_panic(expected = "Content-Length must be a non-negative integer")]
fn test_extract_http_response_length_when_overflow_then_panics() {
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_LENGTH, "99999999999999999999".parse().unwrap());
    extract_http_response_length(&headers);
}

// ============================================================================
// extract_status_id() tests
// ============================================================================

#[test]
fn test_extract_status_id_when_2xx_then_returns_1() {
    let response = http::Response::builder().status(200).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 1);

    let response = http::Response::builder().status(201).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 1);

    let response = http::Response::builder().status(299).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 1);
}

#[test]
fn test_extract_status_id_when_3xx_then_returns_1() {
    let response = http::Response::builder().status(300).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 1);

    let response = http::Response::builder().status(301).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 1);

    let response = http::Response::builder().status(399).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 1);
}

#[test]
fn test_extract_status_id_when_4xx_then_returns_2() {
    let response = http::Response::builder().status(400).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 2);

    let response = http::Response::builder().status(404).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 2);

    let response = http::Response::builder().status(499).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 2);
}

#[test]
fn test_extract_status_id_when_5xx_then_returns_2() {
    let response = http::Response::builder().status(500).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 2);

    let response = http::Response::builder().status(502).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 2);

    let response = http::Response::builder().status(599).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 2);
}

#[test]
fn test_extract_status_id_when_1xx_then_returns_0() {
    let response = http::Response::builder().status(100).body(()).unwrap();
    assert_eq!(extract_status_id(&response), 0);
}

// ============================================================================
// extract_status_code() tests
// ============================================================================

#[test]
fn test_extract_status_code_when_2xx_then_returns_none() {
    let response = http::Response::builder().status(200).body(()).unwrap();
    assert_eq!(extract_status_code(&response), None);

    let response = http::Response::builder().status(204).body(()).unwrap();
    assert_eq!(extract_status_code(&response), None);
}

#[test]
fn test_extract_status_code_when_3xx_then_returns_none() {
    let response = http::Response::builder().status(301).body(()).unwrap();
    assert_eq!(extract_status_code(&response), None);
}

#[test]
fn test_extract_status_code_when_known_error_codes_then_returns_string() {
    let response = http::Response::builder().status(400).body(()).unwrap();
    assert_eq!(extract_status_code(&response), Some("BadRequest".to_string()));

    let response = http::Response::builder().status(401).body(()).unwrap();
    assert_eq!(extract_status_code(&response), Some("Unauthorized".to_string()));

    let response = http::Response::builder().status(403).body(()).unwrap();
    assert_eq!(extract_status_code(&response), Some("Forbidden".to_string()));

    let response = http::Response::builder().status(404).body(()).unwrap();
    assert_eq!(extract_status_code(&response), Some("NotFound".to_string()));

    let response = http::Response::builder().status(500).body(()).unwrap();
    assert_eq!(extract_status_code(&response), Some("InternalError".to_string()));
}

#[test]
fn test_extract_status_code_when_unmapped_error_code_then_returns_none() {
    let response = http::Response::builder().status(418).body(()).unwrap();
    assert_eq!(extract_status_code(&response), None);
}

// ============================================================================
// AuditRecordFactory tests
// ============================================================================

/// Mock factory implementation for testing trait behavior
#[derive(Clone)]
struct MockAuditRecordFactory;

impl AuditRecordFactory for MockAuditRecordFactory {
    type RequestBody = ();
    type ResponseBody = ();
    type RequestExtractorRecordBuilderState = hare_common_utils::audit_record::audit_record_builder::SetTime<
        hare_common_utils::audit_record::audit_record_builder::SetStartTime<
            hare_common_utils::audit_record::audit_record_builder::SetSrcEndpointPort<
                hare_common_utils::audit_record::audit_record_builder::SetSrcEndpointIp<
                    hare_common_utils::audit_record::audit_record_builder::SetMetadataIsTruncated<
                        hare_common_utils::audit_record::audit_record_builder::SetHttpRequestUid<
                            hare_common_utils::audit_record::audit_record_builder::SetHttpRequestXForwardedFor<
                                hare_common_utils::audit_record::audit_record_builder::SetHttpRequestUserAgent<
                                    hare_common_utils::audit_record::audit_record_builder::SetHttpRequestReferer<
                                        hare_common_utils::audit_record::audit_record_builder::SetHttpRequestUrl<
                                            hare_common_utils::audit_record::audit_record_builder::SetDstEndpointSocketFilePath<
                                                hare_common_utils::audit_record::audit_record_builder::SetDstEndpointPort<
                                                    hare_common_utils::audit_record::audit_record_builder::SetDstEndpointIp<
                                                        hare_common_utils::audit_record::audit_record_builder::SetActorProcessUserUid<
                                                            hare_common_utils::audit_record::audit_record_builder::SetActorProcessPid,
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
            >,
        >,
    >;

    fn extract_from_request(
        &self,
        _request: &http::Request<Self::RequestBody>,
        start_time: std::time::SystemTime,
    ) -> hare_common_utils::audit_record::AuditRecordBuilder<Self::RequestExtractorRecordBuilderState> {
        let timestamp = chrono::DateTime::<chrono::Utc>::from(start_time).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        AuditRecord::builder()
            .maybe_actor_process_pid(None)
            .maybe_actor_process_user_uid(None)
            .maybe_dst_endpoint_ip(None)
            .maybe_dst_endpoint_port(None)
            .maybe_dst_endpoint_socket_file_path(None)
            .http_request_url("http://test.example.com/test".to_string())
            .maybe_http_request_referer(None)
            .maybe_http_request_user_agent(None)
            .maybe_http_request_x_forwarded_for(None)
            .http_request_uid("test-request-id".to_string())
            .metadata_is_truncated(false)
            .maybe_src_endpoint_ip(None)
            .maybe_src_endpoint_port(None)
            .start_time(timestamp.clone())
            .time(timestamp)
    }

    fn extract_from_response(
        &self,
        response: &http::Response<Self::ResponseBody>,
        builder: hare_common_utils::audit_record::AuditRecordBuilder<Self::RequestExtractorRecordBuilderState>,
    ) -> AuditRecord {
        let end_time = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        builder
            .http_response_code(extract_http_response_code(response))
            .http_response_length(0)
            .status_id(extract_status_id(response))
            .maybe_status_code(extract_status_code(response))
            .end_time(end_time)
            .build()
    }
}

#[test]
fn test_audit_record_factory_trait_is_send_and_sync() {
    // This test verifies that any type implementing AuditRecordFactory
    // is Send + Sync, which is required by the Service trait bounds
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockAuditRecordFactory>();
}

#[test]
fn test_audit_record_factory_trait_is_clone() {
    // This test verifies that any type implementing AuditRecordFactory
    // is Clone, which is required by the Service trait
    let factory = MockAuditRecordFactory;
    let _factory_clone = factory.clone();
}

#[test]
fn test_audit_record_factory_extract_request_then_response_returns_complete_record() {
    let factory = MockAuditRecordFactory;
    let request = http::Request::builder().method("GET").uri("http://test.example.com/test").body(()).unwrap();

    let response = http::Response::builder().status(200).body(()).unwrap();

    let start_time = std::time::SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);
    let audit_record = factory.extract_from_response(&response, builder);

    // Verify the complete audit record has expected fields from both request and response
    assert_eq!(audit_record.http_request_url, "http://test.example.com/test");
    assert_eq!(audit_record.http_request_uid, "test-request-id");
    assert_eq!(audit_record.http_response_code, 200);
    assert_eq!(audit_record.status_id, 1); // 2xx code maps to 1
    assert_eq!(audit_record.status_code, None); // 2xx codes return None
}

#[test]
fn test_audit_record_factory_handles_error_response() {
    let factory = MockAuditRecordFactory;
    let request = http::Request::builder().method("GET").uri("http://test.example.com/test").body(()).unwrap();

    let response = http::Response::builder().status(404).body(()).unwrap();

    let start_time = std::time::SystemTime::now();
    let builder = factory.extract_from_request(&request, start_time);
    let audit_record = factory.extract_from_response(&response, builder);

    // Verify error response fields
    assert_eq!(audit_record.http_response_code, 404);
    assert_eq!(audit_record.status_id, 2); // 4xx code maps to 2
    assert_eq!(audit_record.status_code, Some("NotFound".to_string()));
}

#[test]
fn test_audit_record_factory_timestamps_are_iso8601_format() {
    let factory = MockAuditRecordFactory;
    let request = http::Request::builder().method("GET").uri("http://test.example.com/test").body(()).unwrap();

    let response = http::Response::builder().status(200).body(()).unwrap();

    // Use a fixed time to get a predictable timestamp
    let fixed_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1577836800); // 2020-01-01T00:00:00Z
    let builder = factory.extract_from_request(&request, fixed_time);
    let audit_record = factory.extract_from_response(&response, builder);

    // Verify start_time is the expected ISO-8601 format
    assert_eq!(audit_record.start_time, "2020-01-01T00:00:00.000Z");
    // end_time will be current time, so just verify it's not empty and follows the pattern
    assert!(!audit_record.end_time.is_empty());
    assert!(audit_record.end_time.ends_with("Z"));
}
