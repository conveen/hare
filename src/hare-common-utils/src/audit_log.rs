/// Audit logging middleware using a factory pattern.
///
/// This is a redesigned version of the audit logging middleware that uses a simpler,
/// clearer factory-based approach instead of the trait-based extractor pattern.
///
/// The key insight is that the middleware needs to store a factory struct
/// that can create audit records, not a type parameter for an extractor trait.
use crate::audit_record::AuditRecord;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;
use valuable::Valuable;

/// Factory trait for creating audit records.
///
/// Implementations of this trait are responsible for extracting audit fields
/// from HTTP requests and responses and building complete audit records using
/// the builder pattern.
///
/// The factory is called twice per request-response cycle:
/// 1. Before the inner service is called, to extract request metadata
/// 2. After the response is received, to extract response metadata and finalize
pub trait AuditRecordFactory: Clone + Send + Sync + 'static {
    /// HTTP request body type this factory works with.
    type RequestBody;
    /// HTTP response body type this factory works with.
    type ResponseBody;
    /// Type state of the audit record builder after extracting request data.
    type RequestExtractorRecordBuilderState: crate::audit_record::audit_record_builder::State;

    /// Extract audit data from the request and populate an audit record builder.
    ///
    /// This is called before passing the request to the inner service, allowing
    /// extraction of request metadata without having to clone the request.
    ///
    /// # Arguments
    /// * `request` - The HTTP request with the factory's specific body type
    /// * `start_time` - When the request was received (for timestamp calculation)
    ///
    /// # Returns
    /// An `AuditRecordBuilder` with request-level fields populated
    fn extract_from_request(
        &self,
        request: &http::Request<Self::RequestBody>,
        start_time: std::time::SystemTime,
    ) -> crate::audit_record::AuditRecordBuilder<Self::RequestExtractorRecordBuilderState>;

    /// Complete the audit record by extracting response data.
    ///
    /// This is called after receiving the response from the inner service.
    /// It takes the builder from `extract_from_request`, populates response-level
    /// fields, and returns the complete audit record.
    ///
    /// # Arguments
    /// * `response` - The HTTP response with the factory's specific body type
    /// * `builder` - The audit record builder from `extract_from_request`
    ///
    /// # Returns
    /// A complete `AuditRecord` with all fields populated
    fn extract_from_response(
        &self,
        response: &http::Response<Self::ResponseBody>,
        builder: crate::audit_record::AuditRecordBuilder<Self::RequestExtractorRecordBuilderState>,
    ) -> AuditRecord;
}

/// Audit logging middleware for Tower services.
///
/// This middleware wraps a service and automatically generates audit records
/// for each request-response pair. The factory is responsible for extracting
/// service-specific fields (like endpoint IPs, TLS info, etc.).
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Clone, Default)]
/// pub struct MyAuditFactory;
///
/// impl AuditRecordFactory for MyAuditFactory {
///     type RequestBody = axum::body::Body;
///     type ResponseBody = axum::body::Body;
///
///     fn extract_from_request(
///         &self,
///         request: &http::Request<Self::RequestBody>,
///         start_time: std::time::SystemTime,
///     ) -> AuditRecord::Builder {
///         AuditRecord::builder()
///             .http_request_url(request.uri().to_string())
///             .dst_endpoint_ip("127.0.0.1".to_string())
///             .src_endpoint_ip(Some("192.168.1.100".to_string()))
///             .src_endpoint_port(Some(54321))
///             .dst_endpoint_port(Some(8001))
///             // ... populate other request fields
///     }
///
///     fn extract_from_response(
///         &self,
///         response: &http::Response<Self::ResponseBody>,
///         builder: AuditRecord::Builder,
///     ) -> AuditRecord {
///         builder
///             .http_response_code(response.status().as_u16())
///             .end_time(/* current timestamp */)
///             // ... populate other response fields
///             .build()
///     }
/// }
///
/// let service = my_service.layer(LogAuditRecordsLayer::new(MyAuditFactory::default()));
/// ```
#[derive(Clone)]
pub struct LogAuditRecords<S, F>
where
    F: AuditRecordFactory,
{
    factory: std::sync::Arc<F>,
    inner: S,
}

impl<S, F> LogAuditRecords<S, F>
where
    F: AuditRecordFactory,
{
    pub fn new(factory: std::sync::Arc<F>, inner: S) -> Self {
        Self { factory, inner }
    }
}

impl<S, F> Service<http::Request<F::RequestBody>> for LogAuditRecords<S, F>
where
    F: AuditRecordFactory,
    F::RequestBody: Send,
    S: Service<http::Request<F::RequestBody>, Response = http::Response<F::ResponseBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: http::Request<F::RequestBody>) -> Self::Future {
        let start_time = std::time::SystemTime::now();
        let factory = self.factory.clone();
        let mut inner = self.inner.clone();

        // Extract request-level audit data before passing to inner service
        let builder = factory.extract_from_request(&request, start_time);

        Box::pin(async move {
            let response = inner.call(request).await?;

            // Extract response-level audit data and finalize the record
            let audit_record = factory.extract_from_response(&response, builder);

            // Log the audit record using tracing
            tracing::info!(record_type = "audit", audit_record = audit_record.as_value());

            Ok(response)
        })
    }
}

/// Layer for audit logging middleware.
///
/// This layer wraps a service with the LogAuditRecords middleware,
/// automatically generating audit records for each request.
#[derive(Clone)]
pub struct LogAuditRecordsLayer<F>
where
    F: AuditRecordFactory,
{
    factory: std::sync::Arc<F>,
}

impl<F> LogAuditRecordsLayer<F>
where
    F: AuditRecordFactory,
{
    pub fn new(factory: std::sync::Arc<F>) -> Self {
        Self { factory }
    }
}

impl<S, F> Layer<S> for LogAuditRecordsLayer<F>
where
    F: AuditRecordFactory,
{
    type Service = LogAuditRecords<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        LogAuditRecords::new(self.factory.clone(), inner)
    }
}

/// Maximum number of header values to capture for header-related fields.
/// Prevents header stuffing DoS attacks.
const MAXIMUM_HEADER_COUNT: usize = 3;

/// Maximum header length in bytes before truncating the value.
/// Prevents header stuffing DoS attacks.
const MAXIMUM_HEADER_VALUE_LENGTH: usize = 1024;

/// Position of header to extract when multiple headers with the same name exist.
#[derive(Debug)]
pub enum HeaderPosition {
    /// Extract all header values
    All,
    /// Extract the first header value
    First,
    /// Extract the last header value
    Last,
}

impl Default for HeaderPosition {
    fn default() -> Self {
        HeaderPosition::All
    }
}

fn extract_header_values_with_truncation<'a>(
    header_values: impl Iterator<Item = &'a http::header::HeaderValue>,
) -> (Option<Vec<String>>, bool) {
    let mut was_truncated = false;

    let values: Vec<String> = header_values
        .filter_map(|v| v.to_str().ok())
        .map(|s| {
            if s.len() > MAXIMUM_HEADER_VALUE_LENGTH {
                was_truncated = true;
                s[..MAXIMUM_HEADER_VALUE_LENGTH].to_string()
            } else {
                s.to_string()
            }
        })
        .collect();

    let values_opt = if values.is_empty() { None } else { Some(values) };
    (values_opt, was_truncated)
}

/// Extract header values from request, limiting count and value length.
///
/// Returns (values, was_truncated) where:
/// - values is None if no headers with the given name are present
/// - was_truncated is true if any header exceeded MAXIMUM_HEADER_VALUE_LENGTH
pub fn extract_header_values(
    headers: &http::HeaderMap,
    header_name: http::header::HeaderName,
    max_count: Option<usize>,
    header_position: Option<HeaderPosition>,
) -> (Option<Vec<String>>, bool) {
    let header_position = header_position.unwrap_or(Default::default());

    match header_position {
        HeaderPosition::All => extract_header_values_with_truncation(
            headers.get_all(header_name).iter().take(max_count.unwrap_or(MAXIMUM_HEADER_COUNT)),
        ),
        HeaderPosition::First => extract_header_values_with_truncation(headers.get_all(header_name).iter().take(1)),
        HeaderPosition::Last => {
            extract_header_values_with_truncation(headers.get_all(header_name).iter().rev().take(1))
        },
    }
}

/// Extract the HTTP response code from a response.
pub fn extract_http_response_code<T>(response: &http::Response<T>) -> u16 {
    response.status().as_u16()
}

/// Extract the HTTP response content length from response headers.
pub fn extract_http_response_length(headers: &http::HeaderMap) -> u64 {
    headers
        .get(http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

/// Determine status ID from HTTP response code.
///
/// Returns:
/// - 1 for 2xx and 3xx (Success)
/// - 2 for 4xx and 5xx (Failure)
/// - 0 for all others (Unknown)
pub fn extract_status_id<T>(response: &http::Response<T>) -> u16 {
    let code = response.status().as_u16();
    match code {
        200..400 => 1,
        400..600 => 2,
        _ => 0,
    }
}

/// Map HTTP status code to OCSF status code string.
///
/// Returns None for 2xx and 3xx status codes (success responses).
pub fn extract_status_code<T>(response: &http::Response<T>) -> Option<String> {
    let code = response.status().as_u16();
    Some(match code {
        400 => "BadRequest".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "NotFound".to_string(),
        405 => "MethodNotAllowed".to_string(),
        406 => "NotAcceptable".to_string(),
        407 => "ProxyAuthenticationRequired".to_string(),
        408 => "RequestTimeout".to_string(),
        409 => "Conflict".to_string(),
        410 => "Gone".to_string(),
        411 => "LengthRequired".to_string(),
        412 => "FailedPrecondition".to_string(),
        413 => "PayloadTooLarge".to_string(),
        414 => "UriTooLong".to_string(),
        415 => "UnsupportedMediaType".to_string(),
        417 => "FailedExpectation".to_string(),
        425 => "TooEarly".to_string(),
        429 => "TooManyRequests".to_string(),
        431 => "HeadersTooLarge".to_string(),
        500 => "InternalError".to_string(),
        _ => return None,
    })
}
