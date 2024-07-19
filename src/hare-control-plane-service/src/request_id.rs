pub const REQUEST_ID_HEADER_NAME: &str = "x-hare-request-id";

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

pub fn request_id_interceptor(mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
    request.extensions_mut().insert(RequestId::default());
    Ok(request)
}
