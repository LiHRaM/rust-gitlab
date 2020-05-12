// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// The state a commit status may have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitStatusState {
    /// The status is expected, but has not started yet.
    Pending,
    /// The status is currently running.
    Running,
    /// The status completed successfully.
    Success,
    /// The status completed with an error.
    Failed,
    /// The status was canceled before it completed.
    Canceled,
}

impl CommitStatusState {
    fn as_str(self) -> &'static str {
        match self {
            CommitStatusState::Pending => "pending",
            CommitStatusState::Running => "running",
            CommitStatusState::Success => "success",
            CommitStatusState::Failed => "failed",
            CommitStatusState::Canceled => "canceled",
        }
    }
}

/// Post a comment on a specific commit in a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateCommitStatus<'a> {
    /// The project to get a commit from.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The commit to comment on.
    #[builder(setter(into))]
    commit: Cow<'a, str>,
    /// The state of the commit status.
    #[builder(setter(into))]
    state: CommitStatusState,

    /// The name of the status.
    #[builder(setter(into), default)]
    name: Option<Cow<'a, str>>,
    /// The name of the ref for the commit.
    #[builder(setter(into), default)]
    ref_: Option<Cow<'a, str>>,
    /// The URL to use for more details.
    #[builder(setter(into), default)]
    target_url: Option<Cow<'a, str>>,
    /// A description for the status.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// The total code coverage (as a percentage).
    #[builder(default)]
    coverage: Option<f64>,
    /// The ID of the pipeline to use (in case it is ambiguous).
    #[builder(default)]
    pipeline_id: Option<u64>,
}

impl<'a> CreateCommitStatus<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateCommitStatusBuilder<'a> {
        CreateCommitStatusBuilder::default()
    }
}

impl<'a> Endpoint for CreateCommitStatus<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            // XXX(gitlab): This is a really bad name for this endpoint. It's the only one that
            // exists in this namespace.
            // https://gitlab.com/gitlab-org/gitlab/-/issues/217412
            "projects/{}/statuses/{}",
            self.project,
            common::path_escaped(&self.commit),
        )
        .into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        pairs.append_pair("state", self.state.as_str());

        self.name
            .as_ref()
            .map(|value| pairs.append_pair("name", value));
        self.ref_
            .as_ref()
            .map(|value| pairs.append_pair("ref", value));
        self.target_url
            .as_ref()
            .map(|value| pairs.append_pair("target_url", value));
        self.description
            .as_ref()
            .map(|value| pairs.append_pair("description", value));
        self.coverage
            .map(|value| pairs.append_pair("coverage", &format!("{}", value)));
        self.pipeline_id
            .map(|value| pairs.append_pair("pipeline_id", &format!("{}", value)));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::repository::commits::{CommitStatusState, CreateCommitStatus};

    #[test]
    fn commit_status_state_as_str() {
        let items = &[
            (CommitStatusState::Pending, "pending"),
            (CommitStatusState::Running, "running"),
            (CommitStatusState::Success, "success"),
            (CommitStatusState::Failed, "failed"),
            (CommitStatusState::Canceled, "canceled"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn project_commit_and_state_are_necessary() {
        let err = CreateCommitStatus::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateCommitStatus::builder()
            .commit("master")
            .state(CommitStatusState::Pending)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn commit_is_necessary() {
        let err = CreateCommitStatus::builder()
            .project(1)
            .state(CommitStatusState::Pending)
            .build()
            .unwrap_err();
        assert_eq!(err, "`commit` must be initialized");
    }

    #[test]
    fn state_is_necessary() {
        let err = CreateCommitStatus::builder()
            .project(1)
            .commit("master")
            .build()
            .unwrap_err();
        assert_eq!(err, "`state` must be initialized");
    }

    #[test]
    fn project_commit_and_state_are_sufficient() {
        CreateCommitStatus::builder()
            .project(1)
            .commit("master")
            .state(CommitStatusState::Pending)
            .build()
            .unwrap();
    }
}
