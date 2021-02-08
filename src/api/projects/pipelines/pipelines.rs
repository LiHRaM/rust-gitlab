// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;

use crate::api::common::{NameOrId, SortOrder};
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

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

impl ParamValue<'static> for PipelineScope {
    fn as_value(&self) -> Cow<'static, str> {
        self.as_str().into()
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
    /// Pipelines which have been scheduled.
    Scheduled,
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
            PipelineStatus::Scheduled => "scheduled",
        }
    }
}

impl ParamValue<'static> for PipelineStatus {
    fn as_value(&self) -> Cow<'static, str> {
        self.as_str().into()
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

impl ParamValue<'static> for PipelineOrderBy {
    fn as_value(&self) -> Cow<'static, str> {
        self.as_str().into()
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
    #[builder(setter(into), default)]
    ref_: Option<Cow<'a, str>>,
    /// Filter pipelines for a given commit SHA.
    #[builder(setter(into), default)]
    sha: Option<Cow<'a, str>>,
    /// Filter pipelines with or without YAML errors.
    #[builder(default)]
    yaml_errors: Option<bool>,
    /// Filter pipelines by the name of the triggering user.
    #[builder(setter(into), default)]
    name: Option<Cow<'a, str>>,
    /// Filter pipelines by the username of the triggering user.
    #[builder(setter(into), default)]
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

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("scope", self.scope)
            .push_opt("status", self.status)
            .push_opt("ref", self.ref_.as_ref())
            .push_opt("sha", self.sha.as_ref())
            .push_opt("yaml_errors", self.yaml_errors)
            .push_opt("name", self.name.as_ref())
            .push_opt("username", self.username.as_ref())
            .push_opt("updated_after", self.updated_after)
            .push_opt("updated_before", self.updated_before)
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort);

        params
    }
}

impl<'a> Pageable for Pipelines<'a> {}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::api::common::SortOrder;
    use crate::api::projects::pipelines::{
        PipelineOrderBy, PipelineScope, PipelineStatus, Pipelines,
    };
    use crate::api::{self, Query};
    use crate::test::client::{ExpectedUrl, SingleTestClient};

    #[test]
    fn pipeline_scope_as_str() {
        let items = &[
            (PipelineScope::Running, "running"),
            (PipelineScope::Pending, "pending"),
            (PipelineScope::Finished, "finished"),
            (PipelineScope::Branches, "branches"),
            (PipelineScope::Tags, "tags"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn pipeline_status_as_str() {
        let items = &[
            (PipelineStatus::Running, "running"),
            (PipelineStatus::Pending, "pending"),
            (PipelineStatus::Success, "success"),
            (PipelineStatus::Failed, "failed"),
            (PipelineStatus::Canceled, "canceled"),
            (PipelineStatus::Skipped, "skipped"),
            (PipelineStatus::Created, "created"),
            (PipelineStatus::Manual, "manual"),
            (PipelineStatus::Scheduled, "scheduled"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn order_by_default() {
        assert_eq!(PipelineOrderBy::default(), PipelineOrderBy::Id);
    }

    #[test]
    fn order_by_as_str() {
        let items = &[
            (PipelineOrderBy::Id, "id"),
            (PipelineOrderBy::Status, "status"),
            (PipelineOrderBy::Ref, "ref"),
            (PipelineOrderBy::UpdatedAt, "updated_at"),
            (PipelineOrderBy::UserId, "user_id"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn project_is_needed() {
        let err = Pipelines::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Pipelines::builder().project(1).build().unwrap();
    }

    #[test]
    fn endpoint() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/simple%2Fproject/pipelines")
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project("simple/project")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_scope() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("scope", "finished")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .scope(PipelineScope::Finished)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_status() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("status", "failed")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .status(PipelineStatus::Failed)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_ref() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("ref", "master")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .ref_("master")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_sha() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("sha", "0000000000000000000000000000000000000000")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .sha("0000000000000000000000000000000000000000")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_yaml_errors() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("yaml_errors", "true")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .yaml_errors(true)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_name() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("name", "name")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .name("name")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_username() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("username", "name")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .username("name")
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_updated_before() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("updated_before", "2020-01-01T00:00:00Z")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .updated_before(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0))
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_updated_after() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("updated_after", "2020-01-01T00:00:00Z")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .updated_after(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0))
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_order_by() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("order_by", "updated_at")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .order_by(PipelineOrderBy::UpdatedAt)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_sort() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("projects/1/pipelines")
            .add_query_params(&[("sort", "desc")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = Pipelines::builder()
            .project(1)
            .sort(SortOrder::Descending)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }
}
