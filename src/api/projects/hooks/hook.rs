// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for a webhook within a project.
#[derive(Debug, Builder)]
pub struct Hook<'a> {
    /// The project to query for webhooks.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the hook.
    hook: u64,
}

impl<'a> Hook<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> HookBuilder<'a> {
        HookBuilder::default()
    }
}

impl<'a> Endpoint for Hook<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/hooks/{}", self.project, self.hook).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::hooks::Hook;

    #[test]
    fn project_and_hook_are_needed() {
        let err = Hook::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = Hook::builder().hook(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn hook_is_needed() {
        let err = Hook::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`hook` must be initialized");
    }

    #[test]
    fn project_and_hook_are_sufficient() {
        Hook::builder().project(1).hook(1).build().unwrap();
    }
}
