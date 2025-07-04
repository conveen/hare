use crate::error;

/// Validate URL according to RFC 1808.
///
/// URL must be valid according to the RFC 1808 specification _and_
/// point to a network location with the ``http`` or `https`` scheme.
/// See https://datatracker.ietf.org/doc/html/rfc1808 for more information.
/// If no scheme provided, will default to ``https``.
///
/// # Returns
///
/// URL with scheme added if needed.
pub fn validate_url(raw_url: &str) -> error::DataPlaneResult<String> {
    match url::Url::parse(raw_url) {
        // url crate requires a scheme, so if URl parses then must have one
        Ok(parsed_url) => {
            if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
                Err(error::DataPlaneError::InvalidUrl { message: format!("invalid scheme {}", parsed_url.scheme()) })
            } else {
                Ok(raw_url.to_string())
            }
        },
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            let mut new_url = raw_url.to_string();
            new_url.insert_str(0, "https://");
            Ok(new_url)
        },
        Err(err) => Err(error::DataPlaneError::InvalidUrl { message: err.to_string() }),
    }
}

/// Parse number of positional arguments from URL.
///
/// URL can have at most one positional argument with placeholder `{}`.
///
/// # Returns
///
/// `1` if `{}` is found anywhere in the string, otherwise `0`.
pub fn gen_num_params_from_url(url: &str) -> i32 {
    if url.contains("{}") {
        1
    } else {
        0
    }
}
