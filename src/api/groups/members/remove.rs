// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Remove a user from a group.
#[derive(Debug, Builder)]
pub struct RemoveGroupMember<'a> {
    /// The group to remove the user from.
    #[builder(setter(into))]
    group: NameOrId<'a>,
    /// The user to remove from the group.
    user: u64,
}

impl<'a> RemoveGroupMember<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> RemoveGroupMemberBuilder<'a> {
        RemoveGroupMemberBuilder::default()
    }
}

impl<'a> Endpoint for RemoveGroupMember<'a> {
    fn method(&self) -> Method {
        Method::DELETE
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("groups/{}/members/{}", self.group, self.user).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::groups::members::RemoveGroupMember;

    #[test]
    fn all_parameters_are_needed() {
        let err = RemoveGroupMember::builder().build().unwrap_err();
        assert_eq!(err, "`group` must be initialized");
    }

    #[test]
    fn group_is_necessary() {
        let err = RemoveGroupMember::builder().user(1).build().unwrap_err();
        assert_eq!(err, "`group` must be initialized");
    }

    #[test]
    fn user_is_necessary() {
        let err = RemoveGroupMember::builder().group(1).build().unwrap_err();
        assert_eq!(err, "`user` must be initialized");
    }

    #[test]
    fn sufficient_parameters() {
        RemoveGroupMember::builder()
            .group("group")
            .user(1)
            .build()
            .unwrap();
    }
}
