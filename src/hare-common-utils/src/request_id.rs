pub const REQUEST_ID_HEADER_NAME: &str = "x-request-id";

/// Assign unique IDs to requests.
///
/// Uses the v4 GUID format.
#[derive(Debug, Clone)]
pub struct RequestId {
    inner: String,
}

impl Default for RequestId {
    fn default() -> Self {
        Self { inner: uuid::Uuid::new_v4().to_string() }
    }
}

impl RequestId {
    pub fn get_id(&self) -> &str {
        self.inner.as_str()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inner)
    }
}

/// [`RequestId`] generator to integrate with [`tower_http::request_id::RequestId`].
#[derive(Debug, Default, Clone)]
pub struct RequestIdGenerator {}

impl RequestIdGenerator {
    /// Generate a new request ID
    pub fn generate_request_id() -> RequestId {
        RequestId::default()
    }
}

impl tower_http::request_id::MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &http::Request<B>) -> Option<tower_http::request_id::RequestId> {
        Some(tower_http::request_id::RequestId::new(Self::generate_request_id().get_id().parse().unwrap()))
    }
}
