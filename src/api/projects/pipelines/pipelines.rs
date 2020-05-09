// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use chrono::{DateTime, Utc};
use derive_builder::Builder;

use crate::api::common::{self, NameOrId, SortOrder};
use crate::api::endpoint_prelude::*;

/// Scopes for pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineScope {
    /// Currently running.
    Running,
    /// Created, but blocked on available runners or triggers.
    Pending,
    /// Completed pipelines.
    Finished,
    /// Pipelines for branches.
    Branches,
    /// Pipelines for tags.
    Tags,
}

impl PipelineScope {
    /// The scope as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            PipelineScope::Running => "running",
            PipelineScope::Pending => "pending",
            PipelineScope::Finished => "finished",
            PipelineScope::Branches => "branches",
            PipelineScope::Tags => "tags",
        }
    }
}

/// The status of a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStatus {
    /// Currently running.
    Running,
    /// Ready to run, but no jobs have been claimed by a runner.
    Pending,
    /// Successfully completed.
    Success,
    /// Unsuccessfully completed.
    Failed,
    /// Canceled.
    Canceled,
    /// Skipped.
    Skipped,
    /// Created, but blocked on available runners or triggers.
    Created,
    /// Awaiting manual triggering.
    Manual,
}

impl PipelineStatus {
    /// The status as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            PipelineStatus::Running => "running",
            PipelineStatus::Pending => "pending",
            PipelineStatus::Success => "success",
            PipelineStatus::Failed => "failed",
            PipelineStatus::Canceled => "canceled",
            PipelineStatus::Skipped => "skipped",
            PipelineStatus::Created => "created",
            PipelineStatus::Manual => "manual",
        }
    }
}

/// Keys pipeline results may be ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineOrderBy {
    /// Order by the pipeline ID.
    Id,
    /// Order by the status of the pipeline.
    Status,
    /// Order by the ref the pipeline was triggered for.
    Ref,
    /// When the pipeline was last updated.
    UpdatedAt,
    /// The ID of the user that created the pipeline.
    UserId,
}

impl Default for PipelineOrderBy {
    fn default() -> Self {
        PipelineOrderBy::Id
    }
}

impl PipelineOrderBy {
    /// The ordering as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            PipelineOrderBy::Id => "id",
            PipelineOrderBy::Status => "status",
            PipelineOrderBy::Ref => "ref",
            PipelineOrderBy::UpdatedAt => "updated_at",
            PipelineOrderBy::UserId => "user_id",
        }
    }
}

/// Query for pipelines within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Pipelines<'a> {
    /// The project to query for pipelines.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    /// Filter pipelines by its scope.
    #[builder(default)]
    scope: Option<PipelineScope>,
    /// Filter pipelines by its status.
    #[builder(default)]
    status: Option<PipelineStatus>,
    /// Filter pipelines by the owning ref.
    #[builder(default)]
    ref_: Option<Cow<'a, str>>,
    /// Filter pipelines for a given commit SHA.
    #[builder(default)]
    sha: Option<Cow<'a, str>>,
    /// Filter pipelines with or without YAML errors.
    #[builder(default)]
    yaml_errors: Option<bool>,
    /// Filter pipelines by the name of the triggering user.
    #[builder(default)]
    name: Option<Cow<'a, str>>,
    /// Filter pipelines by the username of the triggering user.
    #[builder(default)]
    username: Option<Cow<'a, str>>,

    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<PipelineOrderBy>,
    /// Sort order for resulting pipelines.
    #[builder(default)]
    sort: Option<SortOrder>,

    /// Filter pipelines by the last updated date before this time.
    #[builder(default)]
    updated_before: Option<DateTime<Utc>>,
    /// Filter pipelines by the last updated date after this time.
    #[builder(default)]
    updated_after: Option<DateTime<Utc>>,
}

impl<'a> Pipelines<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> PipelinesBuilder<'a> {
        PipelinesBuilder::default()
    }
}

impl<'a> Endpoint for Pipelines<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/pipelines", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.scope
            .map(|value| pairs.append_pair("scope", value.as_str()));
        self.status
            .map(|value| pairs.append_pair("status", value.as_str()));
        self.ref_
            .as_ref()
            .map(|value| pairs.append_pair("ref", value));
        self.sha
            .as_ref()
            .map(|value| pairs.append_pair("sha", value));
        self.yaml_errors
            .map(|value| pairs.append_pair("yaml_errors", common::bool_str(value)));
        self.name
            .as_ref()
            .map(|value| pairs.append_pair("name", value));
        self.username
            .as_ref()
            .map(|value| pairs.append_pair("username", value));

        self.updated_after.map(|value| {
            pairs.append_pair(
                "updated_after",
                &value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )
        });
        self.updated_before.map(|value| {
            pairs.append_pair(
                "updated_before",
                &value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )
        });

        self.order_by
            .map(|value| pairs.append_pair("order_by", value.as_str()));
        self.sort
            .map(|value| pairs.append_pair("sort", value.as_str()));
    }
}

impl<'a> Pageable for Pipelines<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::Pipelines;

    #[test]
    fn project_is_needed() {
        let err = Pipelines::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Pipelines::builder().project(1).build().unwrap();
    }
}
