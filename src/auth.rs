// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use log::error;
use reqwest::blocking::RequestBuilder;
use reqwest::header::{self, HeaderValue};
use thiserror::Error;

#[derive(Debug, Error)]
// TODO #[non_exhaustive]
pub enum AuthError {
    #[error("header value error: {}", source)]
    HeaderValue {
        #[from]
        source: header::InvalidHeaderValue,
    },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
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
    /// Sets the appropriate header on the request.
    ///
    /// Depending on the token type, this will be either the Private-Auth header
    /// or the Authorization header.
    /// Returns an error if the token string cannot be parsed as a header value.
    pub fn set_header(&self, req: RequestBuilder) -> AuthResult<RequestBuilder> {
        Ok(match self {
            Auth::Token(token) => {
                let mut token_header_value = HeaderValue::from_str(&token)?;
                token_header_value.set_sensitive(true);
                req.header("PRIVATE-TOKEN", token_header_value)
            },
            Auth::OAuth2(token) => {
                let value = format!("Bearer {}", token);
                let mut token_header_value = HeaderValue::from_str(&value)?;
                token_header_value.set_sensitive(true);
                req.header("Authorization", token_header_value)
            },
        })
    }
}
