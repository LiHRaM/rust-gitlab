// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use http::{HeaderMap, HeaderValue};
use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AuthError {
    #[error("header value error: {}", source)]
    HeaderValue {
        #[from]
        source: http::header::InvalidHeaderValue,
    },
}

type AuthResult<T> = Result<T, AuthError>;

/// A Gitlab API token
///
/// Gitlab supports two kinds of tokens
#[derive(Clone)]
pub enum Auth {
    /// A personal access token, obtained through Gitlab user settings
    Token(String),
    /// An OAuth2 token, obtained through the OAuth2 flow
    OAuth2(String),
}

impl Auth {
    /// Adds the appropriate header to a set of headers.
    ///
    /// Depending on the token type, this will be either the Private-Token header
    /// or the Authorization header.
    ///
    /// Returns an error if the token string cannot be parsed as a header value.
    pub fn set_header<'a>(
        &self,
        headers: &'a mut HeaderMap<HeaderValue>,
    ) -> AuthResult<&'a mut HeaderMap<HeaderValue>> {
        match self {
            Auth::Token(token) => {
                let mut token_header_value = HeaderValue::from_str(token)?;
                token_header_value.set_sensitive(true);
                headers.insert("PRIVATE-TOKEN", token_header_value);
            },
            Auth::OAuth2(token) => {
                let value = format!("Bearer {}", token);
                let mut token_header_value = HeaderValue::from_str(&value)?;
                token_header_value.set_sensitive(true);
                headers.insert(http::header::AUTHORIZATION, token_header_value);
            },
        }

        Ok(headers)
    }
}
