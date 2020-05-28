// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::NaiveDate;
use derive_builder::Builder;

use crate::api::common::{AccessLevel, NameOrId};
use crate::api::endpoint_prelude::*;

/// Add a user as a member of a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct AddProjectMember<'a> {
    /// The project to add the user to.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The user to add to the project.
    user: u64,
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
            .push("user_id", self.user)
            .push("access_level", self.access_level.as_u64())
            .push_opt("expires_at", self.expires_at);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::common::AccessLevel;
    use crate::api::projects::members::AddProjectMember;

    #[test]
    fn all_parameters_are_needed() {
        let err = AddProjectMember::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = AddProjectMember::builder()
            .user(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn user_is_necessary() {
        let err = AddProjectMember::builder()
            .project(1)
            .access_level(AccessLevel::Developer)
            .build()
            .unwrap_err();
        assert_eq!(err, "`user` must be initialized");
    }

    #[test]
    fn access_level_is_necessary() {
        let err = AddProjectMember::builder()
            .project(1)
            .user(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`access_level` must be initialized");
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
}
