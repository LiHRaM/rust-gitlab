// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::NaiveDate;
use derive_builder::Builder;

use crate::api::common::{AccessLevel, NameOrId};
use crate::api::endpoint_prelude::*;

/// Add a user as a member of a group.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct AddGroupMember<'a> {
    /// The group to add the user to.
    #[builder(setter(into))]
    group: NameOrId<'a>,
    /// The user to add to the group.
    user: u64,
    /// The access level for the user in the group.
    access_level: AccessLevel,

    /// When the user's access expires.
    #[builder(default)]
    expires_at: Option<NaiveDate>,
}

impl<'a> AddGroupMember<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> AddGroupMemberBuilder<'a> {
        AddGroupMemberBuilder::default()
    }
}

impl<'a> Endpoint for AddGroupMember<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("groups/{}/members", self.group).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("user_id", self.user)
            .push("access_level", self.access_level.as_u64())
            .push_opt("expires_at", self.expires_at);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use http::Method;

    use crate::api::common::AccessLevel;
    use crate::api::groups::members::{AddGroupMember, AddGroupMemberBuilderError};
    use crate::api::{self, Query};
    use crate::test::client::{ExpectedUrl, SingleTestClient};

    #[test]
    fn all_parameters_are_needed() {
        let err = AddGroupMember::builder().build().unwrap_err();
        crate::test::assert_missing_field!(err, AddGroupMemberBuilderError, "group");
    }

    #[test]
    fn group_is_necessary() {
        let err = AddGroupMember::builder()
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, AddGroupMemberBuilderError, "group");
    }

    #[test]
    fn user_is_necessary() {
        let err = AddGroupMember::builder()
            .group(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, AddGroupMemberBuilderError, "user");
    }

    #[test]
    fn access_level_is_necessary() {
        let err = AddGroupMember::builder()
            .group(1)
            .user(1)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, AddGroupMemberBuilderError, "access_level");
    }

    #[test]
    fn sufficient_parameters() {
        AddGroupMember::builder()
            .group("group")
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap();
    }

    #[test]
    fn endpoint() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("groups/group%2Fsubgroup/members")
            .content_type("application/x-www-form-urlencoded")
            .body_str(concat!("user_id=1", "&access_level=30"))
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = AddGroupMember::builder()
            .group("group/subgroup")
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_expires_at() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("groups/group%2Fsubgroup/members")
            .content_type("application/x-www-form-urlencoded")
            .body_str(concat!(
                "user_id=1",
                "&access_level=30",
                "&expires_at=2020-01-01",
            ))
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = AddGroupMember::builder()
            .group("group/subgroup")
            .user(1)
            .access_level(AccessLevel::Developer)
            .expires_at(NaiveDate::from_ymd(2020, 1, 1))
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }
}
