// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query a single member of a project.
#[derive(Debug, Builder)]
pub struct ProjectMember<'a> {
    /// The project to query for membership.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the user.
    user: u64,
}

impl<'a> ProjectMember<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ProjectMemberBuilder<'a> {
        ProjectMemberBuilder::default()
    }
}

impl<'a> Endpoint for ProjectMember<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/members/{}", self.project, self.user).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::members::ProjectMember;

    #[test]
    fn project_and_user_are_needed() {
        let err = ProjectMember::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = ProjectMember::builder().user(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn user_is_needed() {
        let err = ProjectMember::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`user` must be initialized");
    }

    #[test]
    fn project_and_user_are_sufficient() {
        ProjectMember::builder().project(1).user(1).build().unwrap();
    }
}
