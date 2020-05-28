// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeSet;
use std::iter;

use derive_builder::Builder;
use itertools::Itertools;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

#[derive(Debug, Clone)]
enum Assignee {
    Unassigned,
    Id(u64),
    Ids(BTreeSet<u64>),
}

impl Assignee {
    fn add_params<'a>(&'a self, params: &mut FormParams<'a>) {
        match self {
            Assignee::Unassigned => {
                params.push("assignee_ids", "0");
            },
            Assignee::Id(id) => {
                params.push("assignee_id", *id);
            },
            Assignee::Ids(ids) => {
                params.extend(ids.iter().map(|&id| ("assignee_ids[]", id)));
            },
        }
    }
}

/// Create a new merge request on project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateMergeRequest<'a> {
    /// The project to open the merge requset *from*.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The name of the source branch for the merge request.
    #[builder(setter(into))]
    source_branch: Cow<'a, str>,
    /// The name of the target branch for the merge request.
    #[builder(setter(into))]
    target_branch: Cow<'a, str>,
    /// The title for the merge request.
    #[builder(setter(into))]
    title: Cow<'a, str>,

    /// The assignee of the merge request.
    #[builder(setter(name = "_assignee"), default, private)]
    assignee: Option<Assignee>,
    /// The description of the merge request.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// The ID of the target project for the merge request.
    #[builder(default)]
    target_project_id: Option<u64>,
    /// Labels to add to the merge request.
    #[builder(setter(name = "_labels"), default, private)]
    labels: BTreeSet<Cow<'a, str>>,
    /// The ID of the milestone to add the merge request to.
    #[builder(default)]
    milestone_id: Option<u64>,
    /// Whether to remove the source branch once merged or not.
    #[builder(default)]
    remove_source_branch: Option<bool>,
    /// Whether to allow collaboration with maintainers of the target project or not.
    #[builder(default)]
    allow_collaboration: Option<bool>,
    /// Whether to squash the branch when merging or not.
    #[builder(default)]
    squash: Option<bool>,

    /// Whether to allow collaboration with maintainers of the target project or not.
    #[deprecated(note = "use `allow_collaboration` instead")]
    #[builder(default)]
    allow_maintainer_to_push: Option<bool>,
}

impl<'a> CreateMergeRequest<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateMergeRequestBuilder<'a> {
        CreateMergeRequestBuilder::default()
    }
}

impl<'a> CreateMergeRequestBuilder<'a> {
    /// Filter unassigned merge requests.
    pub fn unassigned(&mut self) -> &mut Self {
        self.assignee = Some(Some(Assignee::Unassigned));
        self
    }

    /// Filter merge requests assigned to a user (by ID).
    pub fn assignee(&mut self, assignee: u64) -> &mut Self {
        let assignee = match self.assignee.take() {
            Some(Some(Assignee::Ids(mut set))) => {
                set.insert(assignee);
                Assignee::Ids(set)
            },
            Some(Some(Assignee::Id(old_id))) => {
                let set = [old_id, assignee].iter().copied().collect();
                Assignee::Ids(set)
            },
            _ => Assignee::Id(assignee),
        };
        self.assignee = Some(Some(assignee));
        self
    }

    /// Filter merge requests assigned to a user (by ID).
    pub fn assignees<I>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = u64>,
    {
        let assignee = match self.assignee.take() {
            Some(Some(Assignee::Ids(mut set))) => {
                set.extend(iter);
                Assignee::Ids(set)
            },
            Some(Some(Assignee::Id(old_id))) => {
                let set = iter.chain(iter::once(old_id)).collect();
                Assignee::Ids(set)
            },
            _ => Assignee::Ids(iter.collect()),
        };
        self.assignee = Some(Some(assignee));
        self
    }

    /// Add a label.
    pub fn label<L>(&mut self, label: L) -> &mut Self
    where
        L: Into<Cow<'a, str>>,
    {
        self.labels
            .get_or_insert_with(BTreeSet::new)
            .insert(label.into());
        self
    }

    /// Add multiple labels.
    pub fn labels<I, L>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = L>,
        L: Into<Cow<'a, str>>,
    {
        self.labels
            .get_or_insert_with(BTreeSet::new)
            .extend(iter.map(Into::into));
        self
    }
}

impl<'a> Endpoint for CreateMergeRequest<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/merge_requests", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("source_branch", self.source_branch.as_ref())
            .push("target_branch", self.target_branch.as_ref())
            .push("title", self.title.as_ref())
            .push_opt("description", self.description.as_ref())
            .push_opt("target_project_id", self.target_project_id)
            .push_opt("milestone_id", self.milestone_id)
            .push_opt("remove_source_branch", self.remove_source_branch)
            .push_opt("allow_collaboration", self.allow_collaboration)
            .push_opt("squash", self.squash);

        if !self.labels.is_empty() {
            params.push("labels", format!("{}", self.labels.iter().format(",")));
        }
        if let Some(assignee) = self.assignee.as_ref() {
            assignee.add_params(&mut params);
        }

        #[allow(deprecated)]
        {
            params.push_opt("allow_maintainer_to_push", self.allow_maintainer_to_push);
        }

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::merge_requests::CreateMergeRequest;

    #[test]
    fn project_source_branch_target_branch_and_title_are_necessary() {
        let err = CreateMergeRequest::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateMergeRequest::builder()
            .source_branch("source")
            .target_branch("target")
            .title("title")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn source_branch_is_necessary() {
        let err = CreateMergeRequest::builder()
            .project(1)
            .target_branch("target")
            .title("title")
            .build()
            .unwrap_err();
        assert_eq!(err, "`source_branch` must be initialized");
    }

    #[test]
    fn target_branch_is_necessary() {
        let err = CreateMergeRequest::builder()
            .project(1)
            .source_branch("source")
            .title("title")
            .build()
            .unwrap_err();
        assert_eq!(err, "`target_branch` must be initialized");
    }

    #[test]
    fn title_is_necessary() {
        let err = CreateMergeRequest::builder()
            .project(1)
            .source_branch("source")
            .target_branch("target")
            .build()
            .unwrap_err();
        assert_eq!(err, "`title` must be initialized");
    }

    #[test]
    fn project_source_branch_target_branch_and_title_are_sufficient() {
        CreateMergeRequest::builder()
            .project(1)
            .source_branch("source")
            .target_branch("target")
            .title("title")
            .build()
            .unwrap();
    }
}
