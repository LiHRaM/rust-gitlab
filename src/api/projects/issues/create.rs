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

use crate::api::common::NameOrId;
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

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        if !self.title.is_empty() || self.merge_request_to_resolve_discussions_of.is_none() {
            params.push("title", &self.title);
        }

        params
            .push_opt("iid", self.iid)
            .push_opt("description", self.description.as_ref())
            .push_opt("confidential", self.confidential)
            .extend(
                self.assignee_ids
                    .iter()
                    .map(|&value| ("assignee_ids[]", value)),
            )
            .push_opt("milestone_id", self.milestone_id)
            .push_opt("created_at", self.created_at)
            .push_opt("due_date", self.due_date)
            .push_opt(
                "merge_request_to_resolve_discussions_of",
                self.merge_request_to_resolve_discussions_of,
            )
            .push_opt("discussion_to_resolve", self.discussion_to_resolve.as_ref())
            .push_opt("weight", self.weight)
            .push_opt("epic_id", self.epic_id);

        if !self.labels.is_empty() {
            params.push("labels", format!("{}", self.labels.iter().format(",")));
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
