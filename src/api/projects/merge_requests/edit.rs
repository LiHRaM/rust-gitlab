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
use crate::api::projects::merge_requests::create::Assignee;
use crate::api::ParamValue;

#[derive(Debug, Clone)]
enum MergeRequestLabels<'a> {
    Unlabeled,
    Labeled(BTreeSet<Cow<'a, str>>),
}

impl<'a, 'b: 'a> ParamValue<'a> for &'b MergeRequestLabels<'a> {
    fn as_value(self) -> Cow<'a, str> {
        match self {
            MergeRequestLabels::Unlabeled => "".into(),
            MergeRequestLabels::Labeled(labels) => format!("{}", labels.iter().format(",")).into(),
        }
    }
}

/// States an issue may be set to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeRequestStateEvent {
    /// Close the issue.
    Close,
    /// Reopen a closed issue.
    Reopen,
}

impl MergeRequestStateEvent {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            MergeRequestStateEvent::Close => "close",
            MergeRequestStateEvent::Reopen => "reopen",
        }
    }
}

impl ParamValue<'static> for MergeRequestStateEvent {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// Edit a new merge request on project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct EditMergeRequest<'a> {
    /// The project to open the merge requset *from*.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The merge request to edit.
    merge_request: u64,

    /// The name of the target branch for the merge request.
    #[builder(setter(into), default)]
    target_branch: Option<Cow<'a, str>>,
    /// The title for the merge request.
    #[builder(setter(into), default)]
    title: Option<Cow<'a, str>>,
    /// The assignee of the merge request.
    #[builder(setter(name = "_assignee"), default, private)]
    assignee: Option<Assignee>,
    /// The ID of the milestone to add the merge request to.
    #[builder(default)]
    milestone_id: Option<u64>,
    /// Labels to add to the merge request.
    #[builder(setter(name = "_labels"), default, private)]
    labels: Option<MergeRequestLabels<'a>>,
    /// The description of the merge request.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// Change the state of the merge request.
    #[builder(default)]
    state_event: Option<MergeRequestStateEvent>,
    /// Whether to remove the source branch once merged or not.
    #[builder(default)]
    remove_source_branch: Option<bool>,
    /// Whether to squash the branch when merging or not.
    #[builder(default)]
    squash: Option<bool>,
    /// Whether to lock discussion or not..
    #[builder(default)]
    discussion_locked: Option<bool>,
    /// Whether to allow collaboration with maintainers of the target project or not.
    #[builder(default)]
    allow_collaboration: Option<bool>,

    /// Whether to allow collaboration with maintainers of the target project or not.
    #[deprecated(note = "use `allow_collaboration` instead")]
    #[builder(default)]
    allow_maintainer_to_push: Option<bool>,
}

impl<'a> EditMergeRequest<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EditMergeRequestBuilder<'a> {
        EditMergeRequestBuilder::default()
    }
}

impl<'a> EditMergeRequestBuilder<'a> {
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

    /// Clear all labels
    pub fn remove_labels(&mut self) -> &mut Self {
        self.labels = Some(Some(MergeRequestLabels::Unlabeled));
        self
    }

    /// Add a label.
    pub fn label<L>(&mut self, label: L) -> &mut Self
    where
        L: Into<Cow<'a, str>>,
    {
        let label = label.into();
        let labels = if let Some(Some(MergeRequestLabels::Labeled(mut set))) = self.labels.take() {
            set.insert(label);
            set
        } else {
            let mut set = BTreeSet::new();
            set.insert(label);
            set
        };
        self.labels = Some(Some(MergeRequestLabels::Labeled(labels)));
        self
    }

    /// Add multiple labels.
    pub fn labels<I, L>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = L>,
        L: Into<Cow<'a, str>>,
    {
        let iter = iter.map(Into::into);
        let labels = if let Some(Some(MergeRequestLabels::Labeled(mut set))) = self.labels.take() {
            set.extend(iter);
            set
        } else {
            iter.collect()
        };
        self.labels = Some(Some(MergeRequestLabels::Labeled(labels)));
        self
    }
}

impl<'a> Endpoint for EditMergeRequest<'a> {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/merge_requests/{}",
            self.project, self.merge_request,
        )
        .into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push_opt("target_branch", self.target_branch.as_ref())
            .push_opt("title", self.title.as_ref())
            .push_opt("milestone_id", self.milestone_id)
            .push_opt("labels", self.labels.as_ref())
            .push_opt("description", self.description.as_ref())
            .push_opt("state_event", self.state_event)
            .push_opt("remove_source_branch", self.remove_source_branch)
            .push_opt("squash", self.squash)
            .push_opt("discussion_locked", self.discussion_locked)
            .push_opt("allow_collaboration", self.allow_collaboration);

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
    use crate::api::projects::merge_requests::EditMergeRequest;

    #[test]
    fn project_and_merge_request_are_necessary() {
        let err = EditMergeRequest::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = EditMergeRequest::builder()
            .merge_request(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn merge_request_is_necessary() {
        let err = EditMergeRequest::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`merge_request` must be initialized");
    }

    #[test]
    fn project_and_merge_request_are_sufficient() {
        EditMergeRequest::builder()
            .project(1)
            .merge_request(1)
            .build()
            .unwrap();
    }
}
