// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for an environment within a project.
#[derive(Debug, Builder)]
pub struct Environment<'a> {
    /// The project to query for the environment.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the environment.
    environment: u64,
}

impl<'a> Environment<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EnvironmentBuilder<'a> {
        EnvironmentBuilder::default()
    }
}

impl<'a> Endpoint for Environment<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/environment/{}", self.project, self.environment).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::environments::Environment;

    #[test]
    fn project_and_environment_are_needed() {
        let err = Environment::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = Environment::builder().environment(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn environment_is_needed() {
        let err = Environment::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`environment` must be initialized");
    }

    #[test]
    fn project_and_environment_are_sufficient() {
        Environment::builder()
            .project(1)
            .environment(1)
            .build()
            .unwrap();
    }
}
