// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query a members of a group.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct GroupMembers<'a> {
    /// The group to query for membership.
    #[builder(setter(into))]
    group: NameOrId<'a>,

    /// A search string to filter members by.
    #[builder(setter(into), default)]
    query: Option<Cow<'a, str>>,
    /// A search string to filter members by.
    #[builder(setter(name = "_user_ids"), default, private)]
    user_ids: HashSet<u64>,
}

impl<'a> GroupMembers<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> GroupMembersBuilder<'a> {
        GroupMembersBuilder::default()
    }
}

impl<'a> GroupMembersBuilder<'a> {
    /// Filter results by the given user ID.
    pub fn user_id(&mut self, user_id: u64) -> &mut Self {
        self.user_ids
            .get_or_insert_with(HashSet::new)
            .insert(user_id);
        self
    }

    /// Filter results by the given user IDs.
    pub fn user_ids<I>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = u64>,
    {
        self.user_ids.get_or_insert_with(HashSet::new).extend(iter);
        self
    }
}

impl<'a> Endpoint for GroupMembers<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("groups/{}/members", self.group).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("query", self.query.as_ref())
            .extend(self.user_ids.iter().map(|&value| ("user_ids[]", value)));

        params
    }
}

impl<'a> Pageable for GroupMembers<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::groups::members::GroupMembers;

    #[test]
    fn group_is_needed() {
        let err = GroupMembers::builder().build().unwrap_err();
        assert_eq!(err, "`group` must be initialized");
    }

    #[test]
    fn group_is_sufficient() {
        GroupMembers::builder().group(1).build().unwrap();
    }
}
