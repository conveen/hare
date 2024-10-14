use tower_service::Service;

/// Filter headers from the request.
///
/// Useful if headers should only be set by a service and not callers.
/// For example, a service may want to prevent callers from setting customer request IDs.
#[derive(Debug, Clone)]
pub struct FilterHeaders<S> {
    inner: S,
    headers: Vec<String>,
}

impl<S> FilterHeaders<S> {
    pub fn new(inner: S, headers: Vec<String>) -> Self {
        Self { inner, headers }
    }
}

impl<S, RequestBody, ResponseBody> Service<http::Request<RequestBody>> for FilterHeaders<S>
where
    S: Service<http::Request<RequestBody>, Response = http::Response<ResponseBody>>,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<RequestBody>) -> Self::Future {
        for header in self.headers.iter() {
            if req.headers().contains_key(header) {
                req.headers_mut().remove(header);
            }
        }

        self.inner.call(req)
    }
}

/// Filter headers from the request.
///
/// Applies the [`FilterHeaders`] middleware.
#[derive(Debug, Clone)]
pub struct FilterHeadersLayer {
    headers: Vec<String>,
}

impl FilterHeadersLayer {
    pub fn new(headers: Vec<String>) -> Self {
        FilterHeadersLayer { headers }
    }
}

impl<S> tower_layer::Layer<S> for FilterHeadersLayer {
    type Service = FilterHeaders<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FilterHeaders::new(inner, self.headers.clone())
    }
}
