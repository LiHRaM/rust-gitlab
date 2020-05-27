// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate, Utc};
use derive_builder::Builder;
use itertools::Itertools;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;
use crate::api::ParamValue;

/// States an issue may be set to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueStateEvent {
    /// Close the issue.
    Close,
    /// Reopen a closed issue.
    Reopen,
}

impl IssueStateEvent {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            IssueStateEvent::Close => "close",
            IssueStateEvent::Reopen => "reopen",
        }
    }
}

impl ParamValue<'static> for IssueStateEvent {
    fn as_value(self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

#[derive(Debug, Clone)]
enum IssueAssignees {
    Unassigned,
    Assignees(BTreeSet<u64>),
}

#[derive(Debug, Clone)]
enum IssueLabels<'a> {
    Unlabeled,
    Labeled(BTreeSet<Cow<'a, str>>),
}

impl<'a, 'b: 'a> ParamValue<'a> for &'b IssueLabels<'a> {
    fn as_value(self) -> Cow<'a, str> {
        match self {
            IssueLabels::Unlabeled => "".into(),
            IssueLabels::Labeled(labels) => format!("{}", labels.iter().format(",")).into(),
        }
    }
}

/// Create a new issue on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct EditIssue<'a> {
    /// The project to add the issue to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The internal IID of the issue.
    issue_iid: u64,

    /// The title of the new issue.
    #[builder(setter(into), default)]
    title: Option<Cow<'a, str>>,
    /// The description of the issue.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,

    /// Assignees for the issue.
    #[builder(setter(name = "_assignee_ids"), default, private)]
    assignee_ids: Option<IssueAssignees>,
    /// The ID of a milestone to add the issue to.
    #[builder(default)]
    milestone_id: Option<u64>,
    /// Labels to set on the issue.
    #[builder(setter(name = "_labels"), default, private)]
    labels: Option<IssueLabels<'a>>,
    /// Change the state of the issue.
    #[builder(default)]
    state_event: Option<IssueStateEvent>,
    /// Set the last-updated time for the issue.
    #[builder(default)]
    updated_at: Option<DateTime<Utc>>,
    /// Set the due date for the issue.
    #[builder(default)]
    due_date: Option<NaiveDate>,
    /// Set the weight of the issue.
    #[builder(default)]
    weight: Option<u64>,
    /// Set whether discussion of the issue should be locked or not.
    #[builder(default)]
    discussion_locked: Option<bool>,
    /// The ID of the epic to add the issue to.
    #[builder(default)]
    epic_id: Option<u64>,

    /// The internal ID of the epic to add the issue to.
    #[deprecated(note = "use `epic_id` instead")]
    #[builder(default)]
    epic_iid: Option<u64>,
}

impl<'a> EditIssue<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EditIssueBuilder<'a> {
        EditIssueBuilder::default()
    }
}

impl<'a> EditIssueBuilder<'a> {
    /// Unassign the issue.
    pub fn unassign(&mut self) -> &mut Self {
        self.assignee_ids = Some(Some(IssueAssignees::Unassigned));
        self
    }

    /// Assign the issue to a user.
    pub fn assignee_id(&mut self, assignee: u64) -> &mut Self {
        let assignees =
            if let Some(Some(IssueAssignees::Assignees(mut set))) = self.assignee_ids.take() {
                set.insert(assignee);
                set
            } else {
                let mut set = BTreeSet::new();
                set.insert(assignee);
                set
            };
        self.assignee_ids = Some(Some(IssueAssignees::Assignees(assignees)));
        self
    }

    /// Assigne the issue to a set of users.
    pub fn assignee_ids<I>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = u64>,
    {
        let assignees =
            if let Some(Some(IssueAssignees::Assignees(mut set))) = self.assignee_ids.take() {
                set.extend(iter);
                set
            } else {
                iter.collect()
            };
        self.assignee_ids = Some(Some(IssueAssignees::Assignees(assignees)));
        self
    }

    /// Remove all labels from the issue.
    pub fn remove_labels(&mut self) -> &mut Self {
        self.labels = Some(Some(IssueLabels::Unlabeled));
        self
    }

    /// Add a label to the issue.
    ///
    /// Note that the list of labels sent will replace the set on the instance. This only adds it
    /// to the list of labels to add to the set before sending it to the instance.
    pub fn label<L>(&mut self, label: L) -> &mut Self
    where
        L: Into<Cow<'a, str>>,
    {
        let label = label.into();
        let labels = if let Some(Some(IssueLabels::Labeled(mut set))) = self.labels.take() {
            set.insert(label);
            set
        } else {
            let mut set = BTreeSet::new();
            set.insert(label);
            set
        };
        self.labels = Some(Some(IssueLabels::Labeled(labels)));
        self
    }

    /// Add a set of labels to the issue.
    ///
    /// Note that the list of labels sent will replace the set on the instance. This only adds it
    /// to the list of labels to add to the set before sending it to the instance.
    pub fn labels<I, L>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = L>,
        L: Into<Cow<'a, str>>,
    {
        let iter = iter.into_iter().map(Into::into);
        let labels = if let Some(Some(IssueLabels::Labeled(mut set))) = self.labels.take() {
            set.extend(iter);
            set
        } else {
            iter.collect()
        };
        self.labels = Some(Some(IssueLabels::Labeled(labels)));
        self
    }
}

impl<'a> Endpoint for EditIssue<'a> {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues/{}", self.project, self.issue_iid).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push_opt("title", self.title.as_ref())
            .push_opt("description", self.description.as_ref())
            .push_opt("milestone_id", self.milestone_id)
            .push_opt("labels", self.labels.as_ref())
            .push_opt("state_event", self.state_event)
            .push_opt("updated_at", self.updated_at)
            .push_opt("due_date", self.due_date)
            .push_opt("weight", self.weight)
            .push_opt("discussion_locked", self.discussion_locked)
            .push_opt("epic_id", self.epic_id);

        if let Some(assignees) = self.assignee_ids.as_ref() {
            match assignees {
                IssueAssignees::Unassigned => {
                    params.push("assignee_ids[]", "0");
                },
                IssueAssignees::Assignees(ids) => {
                    params.extend(ids.iter().map(|&value| ("assignee_ids[]", value)));
                },
            }
        }

        #[allow(deprecated)]
        {
            params.push_opt("epic_iid", self.epic_iid);
        }

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::{EditIssue, IssueStateEvent};

    #[test]
    fn issue_state_event_as_str() {
        let items = &[
            (IssueStateEvent::Close, "close"),
            (IssueStateEvent::Reopen, "reopen"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn project_and_iid_are_necessary() {
        let err = EditIssue::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = EditIssue::builder().issue_iid(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn iid_is_necessary() {
        let err = EditIssue::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`issue_iid` must be initialized");
    }

    #[test]
    fn project_and_iid_are_sufficient() {
        EditIssue::builder()
            .project(1)
            .issue_iid(1)
            .build()
            .unwrap();
    }
}
