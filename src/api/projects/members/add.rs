// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::NaiveDate;
use derive_builder::Builder;

use crate::api::common::{AccessLevel, CommaSeparatedList, NameOrId};
use crate::api::endpoint_prelude::*;

/// Add a user as a member of a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct AddProjectMember<'a> {
    /// The project to add the user to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The user to add to the project.
    #[builder(setter(name = "_user"), private)]
    user_ids: CommaSeparatedList<u64>,
    /// The access level for the user in the project.
    access_level: AccessLevel,

    /// When the user's access expires.
    #[builder(default)]
    expires_at: Option<NaiveDate>,
}

impl<'a> AddProjectMember<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> AddProjectMemberBuilder<'a> {
        AddProjectMemberBuilder::default()
    }
}

impl<'a> AddProjectMemberBuilder<'a> {
    /// The user to add (by ID).
    pub fn user(&mut self, user: u64) -> &mut Self {
        self.user_ids
            .get_or_insert_with(CommaSeparatedList::new)
            .push(user);
        self
    }

    /// Add a set of users (by ID).
    pub fn users<I>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = u64>,
    {
        self.user_ids
            .get_or_insert_with(CommaSeparatedList::new)
            .extend(iter);
        self
    }
}

impl<'a> Endpoint for AddProjectMember<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/members", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("user_id", &self.user_ids)
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
    use crate::api::projects::members::{AddProjectMember, AddProjectMemberBuilderError};
    use crate::api::{self, Query};
    use crate::test::client::{ExpectedUrl, SingleTestClient};

    #[test]
    fn all_parameters_are_needed() {
        let err = AddProjectMember::builder().build().unwrap_err();
        crate::test::assert_missing_field!(err, AddProjectMemberBuilderError, "project");
    }

    #[test]
    fn project_is_necessary() {
        let err = AddProjectMember::builder()
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, AddProjectMemberBuilderError, "project");
    }

    #[test]
    fn user_is_necessary() {
        let err = AddProjectMember::builder()
            .project(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, AddProjectMemberBuilderError, "user_ids");
    }

    #[test]
    fn access_level_is_necessary() {
        let err = AddProjectMember::builder()
            .project(1)
            .user(1)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, AddProjectMemberBuilderError, "access_level");
    }

    #[test]
    fn sufficient_parameters() {
        AddProjectMember::builder()
            .project("project")
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap();
    }

    #[test]
    fn endpoint() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("projects/simple%2Fproject/members")
            .content_type("application/x-www-form-urlencoded")
            .body_str(concat!("user_id=1", "&access_level=30"))
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = AddProjectMember::builder()
            .project("simple/project")
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_user_multiple() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("projects/simple%2Fproject/members")
            .content_type("application/x-www-form-urlencoded")
            .body_str(concat!(
                "user_id=1%2C2",
                "&access_level=30",
                "&expires_at=2020-01-01",
            ))
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = AddProjectMember::builder()
            .project("simple/project")
            .user(1)
            .user(2)
            .access_level(AccessLevel::Developer)
            .expires_at(NaiveDate::from_ymd(2020, 1, 1))
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }

    #[test]
    fn endpoint_expires_at() {
        let endpoint = ExpectedUrl::builder()
            .method(Method::POST)
            .endpoint("projects/simple%2Fproject/members")
            .content_type("application/x-www-form-urlencoded")
            .body_str(concat!(
                "user_id=1",
                "&access_level=30",
                "&expires_at=2020-01-01",
            ))
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");

        let endpoint = AddProjectMember::builder()
            .project("simple/project")
            .user(1)
            .access_level(AccessLevel::Developer)
            .expires_at(NaiveDate::from_ymd(2020, 1, 1))
            .build()
            .unwrap();
        api::ignore(endpoint).query(&client).unwrap();
    }
}
