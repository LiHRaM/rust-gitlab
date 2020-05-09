// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::error::Error;

use thiserror::Error;

use crate::api::PaginationError;

/// Errors which may occur when using API endpoints.
#[derive(Debug, Error)]
// TODO #[non_exhaustive]
pub enum ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// The client encountered an error.
    #[error("client error: {}", source)]
    Client {
        /// The client error.
        source: E,
    },
    /// The URL failed to parse.
    #[error("failed to parse url: {}", source)]
    UrlParse {
        /// The source of the error.
        #[from]
        source: url::ParseError,
    },
    /// JSON deserialization from GitLab failed.
    #[error("could not parse JSON response: {}", source)]
    Json {
        /// The source of the error.
        #[from]
        source: serde_json::Error,
    },
    /// GitLab returned an error message.
    #[error("gitlab server error: {}", msg)]
    Gitlab {
        /// The error message from GitLab.
        msg: String,
    },
    /// Failed to parse an expected data type from JSON.
    #[error("could not parse {} data from JSON: {}", typename, source)]
    DataType {
        /// The source of the error.
        source: serde_json::Error,
        /// The name of the type that could not be deserialized.
        typename: &'static str,
    },
    /// An error with pagination occurred.
    #[error("failed to handle for pagination: {}", source)]
    Pagination {
        /// The source of the error.
        #[from]
        source: PaginationError,
    },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
}

impl<E> ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// Create an API error in a client error.
    pub fn client(source: E) -> Self {
        ApiError::Client {
            source,
        }
    }

    pub(crate) fn from_gitlab(value: serde_json::Value) -> Self {
        let msg = value
            .pointer("/message")
            .or_else(|| value.pointer("/error"))
            .and_then(|s| s.as_str())
            .unwrap_or_else(|| "<unknown error>");

        ApiError::Gitlab {
            msg: msg.into(),
        }
    }

    pub(crate) fn data_type<T>(source: serde_json::Error) -> Self {
        ApiError::DataType {
            source,
            typename: any::type_name::<T>(),
        }
    }
}
