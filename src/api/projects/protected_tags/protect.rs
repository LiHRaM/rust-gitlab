// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{NameOrId, ProtectedAccessLevel};
use crate::api::endpoint_prelude::*;

/// Protect a tag or set of tags on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct ProtectTag<'a> {
    /// The project to protect a tag within.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The name or glob of the tag to protect.
    #[builder(setter(into))]
    name: Cow<'a, str>,
    /// The minimum access level required to create the tag.
    #[builder(default)]
    create_access_level: Option<ProtectedAccessLevel>,
}

impl<'a> ProtectTag<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ProtectTagBuilder<'a> {
        ProtectTagBuilder::default()
    }
}

impl<'a> Endpoint for ProtectTag<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/protected_tags", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("name", &self.name)
            .push_opt("create_access_level", self.create_access_level);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use http::Method;

    use crate::api::common::ProtectedAccessLevel;
    use crate::api::projects::protected_tags::ProtectTag;
    use crate::api::{self, Query};
    use crate::test::client::{ExpectedUrl, SingleTestClient};

    #[test]
    fn project_and_name_are_needed() {
        let err = ProtectTag::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_required() {
        let err = ProtectTag::builder().name("1.0").build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn name_is_required() {
        let err = ProtectTag::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn project_and_name_are_sufficient() {
        ProtectTag::builder()
            .project(1)
            .name("1.0")
            .build()
            .unwrap();
    }

    #[test]
    fn endpoint() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("projects/simple%2Fproject/protected_tags")
            .content_type("application/x-www-form-urlencoded")
            .body_str("name=1.0")
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = ProtectTag::builder()
            .project("simple/project")
            .name("1.0")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_create_access_level() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("projects/simple%2Fproject/protected_tags")
            .content_type("application/x-www-form-urlencoded")
            .body_str(concat!("name=1.0", "&create_access_level=40"))
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = ProtectTag::builder()
            .project("simple/project")
            .name("1.0")
            .create_access_level(ProtectedAccessLevel::Maintainer)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }
}
