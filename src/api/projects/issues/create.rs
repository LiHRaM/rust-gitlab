// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use derive_builder::Builder;
use itertools::Itertools;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Create a new issue on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateIssue<'a> {
    /// The project to add the issue to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The title of the new issue.
    ///
    /// Note: this is technically optional if `merge_request_to_resolve_discussions_of` is given,
    /// but to avoid more complicated shenanigans around choosing one or the other, this is always
    /// marked as required. Instead, if `title` is explictly empty and
    /// `merge_request_to_resolve_discussions_of` is given, `title` will not be sent allowing
    /// GitLab to generate the default title.
    #[builder(setter(into))]
    title: Cow<'a, str>,

    /// The internal ID of the issue.
    ///
    /// Requires administrator or owner permissions.
    #[builder(default)]
    iid: Option<u64>,
    /// The description of the new issue.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// Whether the issue is confidential or not.
    #[builder(default)]
    confidential: Option<bool>,
    /// Assignees for the issue.
    #[builder(setter(name = "_assignee_ids"), default, private)]
    assignee_ids: HashSet<u64>,
    /// The ID of the milestone for the issue.
    #[builder(default)]
    milestone_id: Option<u64>,
    /// Labels to add to the issue.
    #[builder(setter(name = "_labels"), default, private)]
    labels: HashSet<Cow<'a, str>>,
    /// The creation date of the issue.
    ///
    /// Requires administrator or owner permissions.
    #[builder(default)]
    created_at: Option<DateTime<Utc>>,
    /// The due date for the issue.
    #[builder(default)]
    due_date: Option<NaiveDate>,
    /// The ID of a merge request for which to resolve the discussions.
    ///
    /// Resolves all open discussions unless `discussion_to_resolve` is also passed.
    #[builder(default)]
    merge_request_to_resolve_discussions_of: Option<u64>,
    /// The ID of the discussion to resolve.
    #[builder(setter(into), default)]
    discussion_to_resolve: Option<Cow<'a, str>>,
    /// The weight of the issue.
    #[builder(default)]
    weight: Option<u64>,
    /// The ID of the epic to add the issue to.
    #[builder(default)]
    epic_id: Option<u64>,

    /// The internal ID of the epic to add the issue to.
    #[deprecated(note = "use `epic_id` instead")]
    #[builder(default)]
    epic_iid: Option<u64>,
}

impl<'a> CreateIssue<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateIssueBuilder<'a> {
        CreateIssueBuilder::default()
    }
}

impl<'a> CreateIssueBuilder<'a> {
    /// Assign the issue to a user.
    pub fn assignee_id(&mut self, assignee: u64) -> &mut Self {
        self.assignee_ids
            .get_or_insert_with(HashSet::new)
            .insert(assignee);
        self
    }

    /// Assign the issue to a set of users.
    pub fn assignee_ids<I>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = u64>,
    {
        self.assignee_ids
            .get_or_insert_with(HashSet::new)
            .extend(iter);
        self
    }

    /// Add a label to the issue.
    pub fn label<L>(&mut self, label: L) -> &mut Self
    where
        L: Into<Cow<'a, str>>,
    {
        self.labels
            .get_or_insert_with(HashSet::new)
            .insert(label.into());
        self
    }

    /// Add a set of labels to the issue.
    pub fn labels<I, L>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = L>,
        L: Into<Cow<'a, str>>,
    {
        self.labels
            .get_or_insert_with(HashSet::new)
            .extend(iter.into_iter().map(Into::into));
        self
    }
}

impl<'a> Endpoint for CreateIssue<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/issues", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        if !self.title.is_empty() || self.merge_request_to_resolve_discussions_of.is_none() {
            pairs.append_pair("title", &self.title);
        }

        self.iid
            .map(|value| pairs.append_pair("iid", &format!("{}", value)));
        self.description
            .as_ref()
            .map(|value| pairs.append_pair("description", value));
        self.confidential
            .map(|value| pairs.append_pair("confidential", common::bool_str(value)));
        pairs.extend_pairs(
            self.assignee_ids
                .iter()
                .map(|value| ("assignee_ids[]", format!("{}", value))),
        );
        self.milestone_id
            .map(|value| pairs.append_pair("milestone_id", &format!("{}", value)));
        if !self.labels.is_empty() {
            pairs.append_pair("labels", &format!("{}", self.labels.iter().format(",")));
        }
        self.created_at.map(|value| {
            pairs.append_pair(
                "created_at",
                &value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )
        });
        self.due_date
            .map(|value| pairs.append_pair("due_date", &format!("{}", value.format("%Y-%m-%d"))));
        self.merge_request_to_resolve_discussions_of.map(|value| {
            pairs.append_pair(
                "merge_request_to_resolve_discussions_of",
                &format!("{}", value),
            )
        });
        self.discussion_to_resolve
            .as_ref()
            .map(|value| pairs.append_pair("discussion_to_resolve", value));
        self.weight
            .map(|value| pairs.append_pair("weight", &format!("{}", value)));
        self.epic_id
            .map(|value| pairs.append_pair("epic_id", &format!("{}", value)));

        #[allow(deprecated)]
        {
            self.epic_iid
                .map(|value| pairs.append_pair("epic_iid", &format!("{}", value)));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::issues::CreateIssue;

    #[test]
    fn project_and_title_are_necessary() {
        let err = CreateIssue::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateIssue::builder().title("title").build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn title_is_necessary() {
        let err = CreateIssue::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`title` must be initialized");
    }

    #[test]
    fn project_and_title_are_sufficient() {
        CreateIssue::builder()
            .project(1)
            .title("title")
            .build()
            .unwrap();
    }
}
