// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Query for a specific commit in a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Commit<'a> {
    /// The project to get a commit from.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The commit to get.
    #[builder(setter(into))]
    commit: Cow<'a, str>,

    /// Include commit stats.
    #[builder(default)]
    stats: Option<bool>,
}

impl<'a> Commit<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CommitBuilder<'a> {
        CommitBuilder::default()
    }
}

impl<'a> Endpoint for Commit<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/repository/commits/{}",
            self.project,
            common::path_escaped(&self.commit),
        )
        .into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.stats
            .map(|value| pairs.append_pair("stats", common::bool_str(value)));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::repository::commits::Commit;

    #[test]
    fn project_and_commit_are_necessary() {
        let err = Commit::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = Commit::builder().commit("master").build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn commit_is_necessary() {
        let err = Commit::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`commit` must be initialized");
    }

    #[test]
    fn project_and_commit_are_sufficient() {
        Commit::builder()
            .project(1)
            .commit("master")
            .build()
            .unwrap();
    }
}
