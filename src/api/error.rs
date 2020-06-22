// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::error::Error;

use thiserror::Error;

use crate::api::PaginationError;

/// Errors which may occur when creating form data.
#[derive(Debug, Error)]
// TODO #[non_exhaustive]
pub enum BodyError {
    /// Body data could not be serialized from form parameters.
    #[error("failed to URL encode form parameters: {}", source)]
    UrlEncoded {
        /// The source of the error.
        #[from]
        source: serde_urlencoded::ser::Error,
    },
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
}

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
    /// Body data could not be created.
    #[error("failed to create form data: {}", source)]
    Body {
        /// The source of the error.
        #[from]
        source: BodyError,
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
    /// GitLab returned an error object.
    #[error("gitlab server error: {:?}", obj)]
    GitlabObject {
        /// The error object from GitLab.
        obj: serde_json::Value,
    },
    /// GitLab returned an HTTP error with JSON we did not recognize.
    #[error("gitlab server error: {:?}", obj)]
    GitlabUnrecognized {
        /// The full object from GitLab.
        obj: serde_json::Value,
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
        let error_value = value
            .pointer("/message")
            .or_else(|| value.pointer("/error"));

        if let Some(error_value) = error_value {
            if let Some(msg) = error_value.as_str() {
                ApiError::Gitlab {
                    msg: msg.into(),
                }
            } else {
                ApiError::GitlabObject {
                    obj: error_value.clone(),
                }
            }
        } else {
            ApiError::GitlabUnrecognized {
                obj: value,
            }
        }
    }

    pub(crate) fn data_type<T>(source: serde_json::Error) -> Self {
        ApiError::DataType {
            source,
            typename: any::type_name::<T>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use thiserror::Error;

    use crate::api::ApiError;

    #[derive(Debug, Error)]
    #[error("my error")]
    enum MyError {}

    #[test]
    fn gitlab_error_error() {
        let obj = json!({
            "error": "error contents",
        });

        let err: ApiError<MyError> = ApiError::from_gitlab(obj);
        if let ApiError::Gitlab {
            msg,
        } = err
        {
            assert_eq!(msg, "error contents");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn gitlab_error_message_string() {
        let obj = json!({
            "message": "error contents",
        });

        let err: ApiError<MyError> = ApiError::from_gitlab(obj);
        if let ApiError::Gitlab {
            msg,
        } = err
        {
            assert_eq!(msg, "error contents");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn gitlab_error_message_object() {
        let err_obj = json!({
            "blah": "foo",
        });
        let obj = json!({
            "message": err_obj,
        });

        let err: ApiError<MyError> = ApiError::from_gitlab(obj);
        if let ApiError::GitlabObject {
            obj,
        } = err
        {
            assert_eq!(obj, err_obj);
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn gitlab_error_message_unrecognized() {
        let err_obj = json!({
            "some_weird_key": "an even weirder value",
        });

        let err: ApiError<MyError> = ApiError::from_gitlab(err_obj.clone());
        if let ApiError::GitlabUnrecognized {
            obj,
        } = err
        {
            assert_eq!(obj, err_obj);
        } else {
            panic!("unexpected error: {}", err);
        }
    }
}
