// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crates::reqwest;
use crates::serde_json::Value;

error_chain! {
    foreign_links {
        Reqwest(reqwest::Error)
            #[doc = "An error from the reqwest crate."];
    }

    errors {
        /// Error occurred when communicating with Gitlab.
        Communication {
            display("communication error")
        }
        /// Header value parsing error; should never occur.
        HeaderValueParse {
            display("header value error")
        }
        /// URL parsing error; should never occur.
        UrlParse {
            display("url error")
        }
        /// Gitlab returned an error message.
        Gitlab(msg: String) {
            display("gitlab error: {}", msg)
        }
        /// Failed to deserialize a Gitlab result into a structure.
        Deserialize {
            display("deserialization error")
        }
    }
}

impl Error {
    /// Extract the message from a Gitlab JSON error.
    pub fn from_gitlab(value: Value) -> Self {
        let msg = value.pointer("/message")
            .and_then(|s| s.as_str())
            .unwrap_or_else(|| "unknown error");

        Error::from_kind(ErrorKind::Gitlab(msg.to_string()))
    }
}
