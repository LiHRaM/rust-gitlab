// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate ease;
use self::ease::Error as EaseError;

extern crate serde_json;
use self::serde_json::Error as SerdeError;
use self::serde_json::Value;

extern crate url;
use self::url::ParseError as UrlError;

use std::error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
/// Errors which may occur when communicating with Gitlab.
pub enum Error {
    /// Error occurred when communicating with Gitlab.
    EaseError(EaseError),
    /// URL parsing error; should never occur.
    UrlError(UrlError),
    /// Gitlab returned an error message.
    GitlabError(String),
    /// Failed to deserialize a Gitlab result into a structure.
    DeserializeError(Box<SerdeError>),
}

impl Error {
    /// Extract the message from a Gitlab JSON error.
    pub fn from_gitlab(value: Value) -> Self {
        let msg = value.pointer("/message")
            .and_then(|s| s.as_string())
            .unwrap_or_else(|| "unknown error");

        Error::GitlabError(msg.to_owned())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::EaseError(ref error) => write!(f, "ease error: {:?}", error),
            Error::UrlError(ref error) => write!(f, "url error: {}", error),
            Error::GitlabError(ref error) => write!(f, "gitlab error: {}", error),
            Error::DeserializeError(ref error) => write!(f, "deserialization error: {}", error),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "GitLab API error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::EaseError(ref error) => Some(error),
            Error::UrlError(ref error) => Some(error),
            Error::DeserializeError(ref error) => Some(error),
            _ => None,
        }
    }
}

impl From<EaseError> for Error {
    fn from(error: EaseError) -> Self {
        Error::EaseError(error)
    }
}

impl From<UrlError> for Error {
    fn from(error: UrlError) -> Self {
        Error::UrlError(error)
    }
}

impl From<SerdeError> for Error {
    fn from(error: SerdeError) -> Self {
        Error::DeserializeError(Box::new(error))
    }
}
