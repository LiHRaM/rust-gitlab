// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Query for statuses on a specific commit in a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CommitStatuses<'a> {
    /// The project to get a commit from.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The commit to get statuses from.
    #[builder(setter(into))]
    commit: Cow<'a, str>,

    /// The branch or tag to check.
    #[builder(setter(into), default)]
    ref_: Option<Cow<'a, str>>,
    /// Filter by build stage.
    #[builder(setter(into), default)]
    stage: Option<Cow<'a, str>>,
    /// Filter by job name.
    #[builder(setter(into), default)]
    name: Option<Cow<'a, str>>,
    /// Return all statuses, not just the latest.
    #[builder(default)]
    all: Option<bool>,
}

impl<'a> CommitStatuses<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CommitStatusesBuilder<'a> {
        CommitStatusesBuilder::default()
    }
}

impl<'a> Endpoint for CommitStatuses<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/repository/commits/{}/statuses",
            self.project,
            common::path_escaped(&self.commit),
        )
        .into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.ref_
            .as_ref()
            .map(|value| pairs.append_pair("ref", value));
        self.stage
            .as_ref()
            .map(|value| pairs.append_pair("stage", value));
        self.name
            .as_ref()
            .map(|value| pairs.append_pair("name", value));
        self.all
            .map(|value| pairs.append_pair("all", common::bool_str(value)));
    }
}

impl<'a> Pageable for CommitStatuses<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::repository::commits::CommitStatuses;

    #[test]
    fn project_and_commit_are_necessary() {
        let err = CommitStatuses::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CommitStatuses::builder()
            .commit("master")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn commit_is_necessary() {
        let err = CommitStatuses::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`commit` must be initialized");
    }

    #[test]
    fn project_and_commit_are_sufficient() {
        CommitStatuses::builder()
            .project(1)
            .commit("master")
            .build()
            .unwrap();
    }
}
