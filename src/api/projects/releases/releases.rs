// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query releases of a project.
#[derive(Debug, Clone, Builder)]
pub struct ProjectReleases<'a> {
    /// The project to query for releases.
    #[builder(setter(into))]
    project: NameOrId<'a>,
}

impl<'a> ProjectReleases<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ProjectReleasesBuilder<'a> {
        ProjectReleasesBuilder::default()
    }
}

impl<'a> Endpoint for ProjectReleases<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/releases", self.project).into()
    }
}

impl<'a> Pageable for ProjectReleases<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::releases::ProjectReleases;
    use crate::api::{self, Query};
    use crate::test::client::{ExpectedUrl, SingleTestClient};

    #[test]
    fn project_is_needed() {
        let err = ProjectReleases::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        ProjectReleases::builder().project(1).build().unwrap();
    }

    #[test]
    fn endpoint() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/project/releases")
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = ProjectReleases::builder()
            .project("project")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }
}
