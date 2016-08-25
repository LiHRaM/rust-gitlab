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

quick_error! {
    #[derive(Debug)]
    /// Errors which may occur when communicating with Gitlab.
    pub enum Error {
        /// Error occurred when communicating with Gitlab.
        Ease(err: EaseError) {
            cause(err)
            display("ease error: {:?}", err)
            from()
        }
        /// URL parsing error; should never occur.
        UrlParse(err: UrlError) {
            cause(err)
            display("url error: {:?}", err)
            from()
        }
        /// Gitlab returned an error message.
        Gitlab(err: String) {
            display("gitlab error: {:?}", err)
        }
        /// Failed to deserialize a Gitlab result into a structure.
        Deserialize(err: Box<SerdeError>) {
            cause(err)
            display("deserialization error: {:?}", err)
            from()
        }
    }
}

impl Error {
    /// Extract the message from a Gitlab JSON error.
    pub fn from_gitlab(value: Value) -> Self {
        let msg = value.pointer("/message")
            .and_then(|s| s.as_string())
            .unwrap_or_else(|| "unknown error");

        Error::Gitlab(msg.to_string())
    }
}

impl From<SerdeError> for Error {
    fn from(error: SerdeError) -> Self {
        Error::Deserialize(Box::new(error))
    }
}
