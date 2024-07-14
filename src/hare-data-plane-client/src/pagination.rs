use serde::{Deserialize, Serialize};

use crate::error;

#[derive(Debug, Deserialize, Serialize)]
pub struct ContinuationToken {
    pub page_size: u32,
    pub offset: u32,
}

impl ContinuationToken {
    pub fn new(page_size: u32, offset: u32) -> Self {
        ContinuationToken { page_size, offset }
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

    pub fn try_to_string<E: base64::engine::Engine>(&self, encoder: &E) -> error::DataPlaneResult<String> {
        Ok(encoder.encode(serde_json::to_vec(&self)?))
    }
}
