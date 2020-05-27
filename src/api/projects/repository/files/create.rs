// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::str;

use derive_builder::Builder;
use log::warn;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

/// Encodings for uploading file contents through an HTTP request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// No special encoding.
    ///
    /// Only supports UTF-8 content.
    Text,
    /// Base64-encoding.
    ///
    /// Supports representing binary content.
    Base64,
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::Text
    }
}

impl Encoding {
    fn is_binary_safe(self) -> bool {
        match self {
            Encoding::Text => false,
            Encoding::Base64 => true,
        }
    }

    fn encode<'a>(self, as_string: Option<&'a str>, content: &'a [u8]) -> Cow<'a, str> {
        match self {
            Encoding::Text => {
                if let Some(string) = as_string {
                    string.into()
                } else {
                    panic!("attempting to encode non-utf8 content using text!");
                }
            },
            Encoding::Base64 => base64::encode(content).into(),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Encoding::Text => "text",
            Encoding::Base64 => "base64",
        }
    }
}

impl ParamValue<'static> for Encoding {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// Create a new file in a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateFile<'a> {
    /// The project to create a file within.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The path to the file in the repository.
    ///
    /// This is automatically escaped as needed.
    #[builder(setter(into))]
    file_path: Cow<'a, str>,
    /// The branch to use for the new commit.
    #[builder(setter(into))]
    branch: Cow<'a, str>,
    /// The content of the new file.
    ///
    /// This will automatically be encoded according to the `encoding` parameter.
    #[builder(setter(into))]
    content: Cow<'a, [u8]>,
    /// The commit message to use.
    #[builder(setter(into))]
    commit_message: Cow<'a, str>,

    /// Where to start the branch from (if it doesn't already exist).
    #[builder(setter(into), default)]
    start_branch: Option<Cow<'a, str>>,
    /// The encoding to use for the content.
    ///
    /// Note that if `text` is requested and `content` contains non-UTF-8 content, a warning will
    /// be generated and a binary-safe encoding used instead.
    #[builder(setter(into), default)]
    encoding: Option<Encoding>,
    /// The email of the author for the new commit.
    #[builder(setter(into), default)]
    author_email: Option<Cow<'a, str>>,
    /// The name of the author for the new commit.
    #[builder(setter(into), default)]
    author_name: Option<Cow<'a, str>>,
}

impl<'a> CreateFile<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateFileBuilder<'a> {
        CreateFileBuilder::default()
    }
}

const SAFE_ENCODING: Encoding = Encoding::Base64;

impl<'a> Endpoint for CreateFile<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/repository/files/{}",
            self.project,
            common::path_escaped(&self.file_path),
        )
        .into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("branch", &self.branch)
            .push("commit_message", &self.commit_message)
            .push_opt("start_branch", self.start_branch.as_ref())
            .push_opt("author_email", self.author_email.as_ref())
            .push_opt("author_name", self.author_name.as_ref());

        let content = str::from_utf8(&self.content);
        let needs_encoding = content.is_err();
        let encoding = self.encoding.unwrap_or_default();
        let actual_encoding = if needs_encoding && !encoding.is_binary_safe() {
            warn!(
                "forcing the encoding to {} due to utf-8 unsafe content",
                SAFE_ENCODING.as_str(),
            );
            SAFE_ENCODING
        } else {
            encoding
        };
        params.push("content", encoding.encode(content.ok(), &self.content));
        self.encoding
            // Use the actual encoding.
            .map(|_| actual_encoding)
            // Force the encoding if we're not using the default.
            .or_else(|| {
                if actual_encoding != Encoding::default() {
                    Some(actual_encoding)
                } else {
                    None
                }
            })
            .map(|value| params.push("encoding", value));

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::repository::files::{CreateFile, Encoding};

    #[test]
    fn encoding_default() {
        assert_eq!(Encoding::default(), Encoding::Text);
    }

    #[test]
    fn encoding_is_binary_safe() {
        let items = &[(Encoding::Text, false), (Encoding::Base64, true)];

        for (i, s) in items {
            assert_eq!(i.is_binary_safe(), *s);
        }
    }

    #[test]
    fn encoding_encode_text() {
        let encoding = Encoding::Text;
        assert_eq!(encoding.encode(Some("foo"), b"foo"), "foo");
    }

    #[test]
    #[should_panic = "attempting to encode non-utf8 content using text!"]
    fn encoding_encode_text_bad() {
        let encoding = Encoding::Text;
        encoding.encode(None, b"\xff");
    }

    #[test]
    fn encoding_encode_base64() {
        let encoding = Encoding::Base64;
        assert_eq!(encoding.encode(None, b"foo"), "Zm9v");
    }

    #[test]
    fn encoding_as_str() {
        let items = &[(Encoding::Text, "text"), (Encoding::Base64, "base64")];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn all_parameters_are_needed() {
        let err = CreateFile::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_required() {
        let err = CreateFile::builder()
            .file_path("new/file")
            .branch("master")
            .commit_message("commit message")
            .content(&b"contents"[..])
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn file_path_is_required() {
        let err = CreateFile::builder()
            .project(1)
            .branch("master")
            .commit_message("commit message")
            .content(&b"contents"[..])
            .build()
            .unwrap_err();
        assert_eq!(err, "`file_path` must be initialized");
    }

    #[test]
    fn branch_is_required() {
        let err = CreateFile::builder()
            .project(1)
            .file_path("new/file")
            .commit_message("commit message")
            .content(&b"contents"[..])
            .build()
            .unwrap_err();
        assert_eq!(err, "`branch` must be initialized");
    }

    #[test]
    fn commit_message_is_required() {
        let err = CreateFile::builder()
            .project(1)
            .file_path("new/file")
            .branch("master")
            .content(&b"contents"[..])
            .build()
            .unwrap_err();
        assert_eq!(err, "`commit_message` must be initialized");
    }

    #[test]
    fn content_is_required() {
        let err = CreateFile::builder()
            .project(1)
            .file_path("new/file")
            .branch("master")
            .commit_message("commit message")
            .build()
            .unwrap_err();
        assert_eq!(err, "`content` must be initialized");
    }

    #[test]
    fn sufficient_parameters() {
        CreateFile::builder()
            .project(1)
            .file_path("new/file")
            .branch("master")
            .commit_message("commit message")
            .content(&b"contents"[..])
            .build()
            .unwrap();
    }
}
