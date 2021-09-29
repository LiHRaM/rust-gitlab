// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{borrow::Cow, collections::BTreeSet, iter};

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::Method;

use crate::api::{
    common::{CommaSeparatedList, NameOrId, SortOrder},
    helpers::{Labels, Milestone, ReactionEmoji},
    Endpoint, Pageable, QueryParams,
};

use super::{
    Assignee, IssueDueDateFilter, IssueIteration, IssueOrderBy, IssueScope, IssueSearchScope,
    IssueState, IssueWeight,
};

/// Query for issues within a group.
///
/// TODO: Negation (not) filters are not yet supported.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct GroupIssues<'a> {
    /// The group to query for issues.
    #[builder(setter(into))]
    group: NameOrId<'a>,

    /// Filter issues with specific internal IDs.
    #[builder(setter(name = "_iids"), default, private)]
    iids: BTreeSet<u64>,
    /// Filter issues based on state.
    #[builder(default)]
    state: Option<IssueState>,
    /// Filter issues based on labels.
    #[builder(setter(name = "_labels"), default, private)]
    labels: Option<Labels<'a>>,
    /// Include label details in the result.
    #[builder(default)]
    with_labels_details: Option<bool>,
    /// Filter by the iteration.
    #[builder(default)]
    iteration: Option<IssueIteration<'a>>,
    /// Filter issues with a milestone.
    #[builder(setter(name = "_milestone"), default, private)]
    milestone: Option<Milestone<'a>>,
    /// Filter issues within a scope.
    #[builder(default)]
    scope: Option<IssueScope>,
    /// Filter issues by author.
    #[builder(setter(into), default)]
    author: Option<NameOrId<'a>>,
    /// Filter issues by assignees.
    #[builder(setter(name = "_assignee"), default, private)]
    assignee: Option<Assignee<'a>>,
    /// Filter issues by the API caller's reactions.
    #[builder(setter(name = "_my_reaction_emoji"), default, private)]
    my_reaction_emoji: Option<ReactionEmoji<'a>>,
    /// Filter issues by weight.
    #[builder(default)]
    weight: Option<IssueWeight>,

    /// Filter issues with a search query.
    #[builder(setter(into), default)]
    search: Option<Cow<'a, str>>,
    #[builder(setter(name = "_search_in"), default, private)]
    search_in: Option<CommaSeparatedList<IssueSearchScope>>,
    /// Filter issues created after a point in time.
    #[builder(default)]
    created_after: Option<DateTime<Utc>>,
    /// Filter issues created before a point in time.
    #[builder(default)]
    created_before: Option<DateTime<Utc>>,
    /// Filter issues last updated after a point in time.
    #[builder(default)]
    updated_after: Option<DateTime<Utc>>,
    /// Filter issues last updated before a point in time.
    #[builder(default)]
    updated_before: Option<DateTime<Utc>>,
    /// Filter issues by confidentiality.
    #[builder(default)]
    confidential: Option<bool>,
    /// Filter issues by due date.
    #[builder(default)]
    due_date: Option<IssueDueDateFilter>,

    // TODO: How best to support this parameter?
    // not
    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<IssueOrderBy>,
    /// The sort order for return results.
    #[builder(default)]
    sort: Option<SortOrder>,
}

impl<'a> GroupIssues<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> GroupIssuesBuilder<'a> {
        GroupIssuesBuilder::default()
    }
}

impl<'a> GroupIssuesBuilder<'a> {
    /// Return an issue with an internal ID.
    pub fn iid(&mut self, iid: u64) -> &mut Self {
        self.iids.get_or_insert_with(BTreeSet::new).insert(iid);
        self
    }

    /// Return issues with one of a set of internal IDs.
    pub fn iids<I>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = u64>,
    {
        self.iids.get_or_insert_with(BTreeSet::new).extend(iter);
        self
    }

    /// Filter unlabeled issues.
    pub fn unlabeled(&mut self) -> &mut Self {
        self.labels = Some(Some(Labels::None));
        self
    }

    /// Filter issues with any label.
    pub fn with_any_label(&mut self) -> &mut Self {
        self.labels = Some(Some(Labels::Any));
        self
    }

    /// Filter issues with a given label.
    pub fn label<L>(&mut self, label: L) -> &mut Self
    where
        L: Into<Cow<'a, str>>,
    {
        let label = label.into();
        let labels = if let Some(Some(Labels::AllOf(mut set))) = self.labels.take() {
            set.push(label);
            set
        } else {
            iter::once(label).collect()
        };
        self.labels = Some(Some(Labels::AllOf(labels)));
        self
    }

    /// Filter issues with all of the given labels.
    pub fn labels<I, L>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = L>,
        L: Into<Cow<'a, str>>,
    {
        let iter = iter.into_iter().map(Into::into);
        let labels = if let Some(Some(Labels::AllOf(mut set))) = self.labels.take() {
            set.extend(iter);
            set
        } else {
            iter.collect()
        };
        self.labels = Some(Some(Labels::AllOf(labels)));
        self
    }

    /// Filter issues without a milestone.
    pub fn without_milestone(&mut self) -> &mut Self {
        self.milestone = Some(Some(Milestone::None));
        self
    }

    /// Filter issues with any milestone.
    pub fn any_milestone(&mut self) -> &mut Self {
        self.milestone = Some(Some(Milestone::Any));
        self
    }

    /// Filter issues with a given milestone.
    pub fn milestone<M>(&mut self, milestone: M) -> &mut Self
    where
        M: Into<Cow<'a, str>>,
    {
        self.milestone = Some(Some(Milestone::Named(milestone.into())));
        self
    }

    /// Filter unassigned issues.
    pub fn unassigned(&mut self) -> &mut Self {
        self.assignee = Some(Some(Assignee::Unassigned));
        self
    }

    /// Filter assigned issues.
    pub fn assigned(&mut self) -> &mut Self {
        self.assignee = Some(Some(Assignee::Assigned));
        self
    }

    /// Filter issues assigned to a user (by ID).
    pub fn assignee_id(&mut self, assignee: u64) -> &mut Self {
        self.assignee = Some(Some(Assignee::Id(assignee)));
        self
    }

    /// Filter issues assigned to a users (by username).
    pub fn assignee<A>(&mut self, assignee: A) -> &mut Self
    where
        A: Into<Cow<'a, str>>,
    {
        let assignee = assignee.into();
        let assignees = if let Some(Some(Assignee::Usernames(mut set))) = self.assignee.take() {
            set.insert(assignee);
            set
        } else {
            let mut set = BTreeSet::new();
            set.insert(assignee);
            set
        };
        self.assignee = Some(Some(Assignee::Usernames(assignees)));
        self
    }

    /// Filter issues assigned to a set of users.
    pub fn assignees<I, A>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = A>,
        A: Into<Cow<'a, str>>,
    {
        let iter = iter.into_iter().map(Into::into);
        let assignees = if let Some(Some(Assignee::Usernames(mut set))) = self.assignee.take() {
            set.extend(iter);
            set
        } else {
            iter.collect()
        };
        self.assignee = Some(Some(Assignee::Usernames(assignees)));
        self
    }

    /// Filter issues without a reaction by the API caller.
    pub fn no_reaction(&mut self) -> &mut Self {
        self.my_reaction_emoji = Some(Some(ReactionEmoji::None));
        self
    }

    /// Filter issues with any reaction by the API caller.
    pub fn any_reaction(&mut self) -> &mut Self {
        self.my_reaction_emoji = Some(Some(ReactionEmoji::Any));
        self
    }

    /// Filter issues with a specific reaction by the API caller.
    pub fn my_reaction<E>(&mut self, emoji: E) -> &mut Self
    where
        E: Into<Cow<'a, str>>,
    {
        self.my_reaction_emoji = Some(Some(ReactionEmoji::Emoji(emoji.into())));
        self
    }

    /// The scopes to look for search query within.
    pub fn search_in(&mut self, scope: IssueSearchScope) -> &mut Self {
        self.search_in
            .get_or_insert(None)
            .get_or_insert_with(CommaSeparatedList::new)
            .push(scope);
        self
    }
}

impl<'a> Endpoint for GroupIssues<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("groups/{}/issues", self.group).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .extend(self.iids.iter().map(|&value| ("iids[]", value)))
            .push_opt("state", self.state)
            .push_opt("labels", self.labels.as_ref())
            .push_opt("with_labels_details", self.with_labels_details)
            .push_opt("milestone", self.milestone.as_ref())
            .push_opt("scope", self.scope)
            .push_opt("my_reaction_emoji", self.my_reaction_emoji.as_ref())
            .push_opt("weight", self.weight)
            .push_opt("search", self.search.as_ref())
            .push_opt("in", self.search_in.as_ref())
            .push_opt("created_after", self.created_after)
            .push_opt("created_before", self.created_before)
            .push_opt("updated_after", self.updated_after)
            .push_opt("updated_before", self.updated_before)
            .push_opt("confidential", self.confidential)
            .push_opt("due_date", self.due_date)
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort);

        if let Some(author) = self.author.as_ref() {
            match author {
                NameOrId::Name(name) => {
                    params.push("author_username", name);
                },
                NameOrId::Id(id) => {
                    params.push("author_id", *id);
                },
            }
        }
        if let Some(iteration) = self.iteration.as_ref() {
            iteration.add_params(&mut params);
        }
        if let Some(assignee) = self.assignee.as_ref() {
            assignee.add_params(&mut params);
        }

        params
    }
}

impl<'a> Pageable for GroupIssues<'a> {}
