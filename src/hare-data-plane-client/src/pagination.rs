use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Debug, Deserialize, Serialize)]
pub struct ContinuationToken<O> {
    pub page_size: u32,
    pub offset: O,
}

impl<O> ContinuationToken<O>
where
    O: Default + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(page_size: u32, offset: Option<O>) -> Self {
        Self { page_size, offset: offset.unwrap_or_default() }
    }

    pub fn try_from_slice<E: base64::engine::Engine, T: AsRef<[u8]>>(
        decoder: &E,
        buffer: T,
    ) -> error::DataPlaneResult<Self> {
        Ok(serde_json::from_slice(&decoder.decode(buffer)?)?)
    }

    pub fn try_from_str<E: base64::engine::Engine, T: AsRef<str>>(
        decoder: &E,
        input: T,
    ) -> error::DataPlaneResult<Self> {
        Self::try_from_slice(decoder, input.as_ref())
    }

    pub fn try_from_pagination_request<E: base64::engine::Engine>(
        decoder: &E,
        pagination_request: &hare_common_model::pagination::PaginationRequest,
    ) -> error::DataPlaneResult<Self> {
        if let Some(continuation_token) = pagination_request.continuation_token.as_ref() {
            Self::try_from_str(decoder, continuation_token)
        } else if let Some(page_size) = pagination_request.page_size {
            Ok(Self::new(page_size as u32, Default::default()))
        } else {
            Err(error::DataPlaneError::InvalidArgument {
                message: "Must supply either page_size or continuation_token".to_string(),
            })
        }
    }

    pub fn try_to_string<E: base64::engine::Engine>(&self, encoder: &E) -> error::DataPlaneResult<String> {
        Ok(encoder.encode(serde_json::to_vec(&self)?))
    }
}
