// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Query for projects on an instance.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Project<'a> {
    /// The project to get.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    /// Include project statistics in the results.
    #[builder(default)]
    statistics: Option<bool>,
    /// Include project license information in the results.
    #[builder(default)]
    license: Option<bool>,
    /// Search for projects with custom attributes.
    #[builder(default)]
    with_custom_attributes: Option<bool>,
}

impl<'a> Project<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> ProjectBuilder<'a> {
        ProjectBuilder::default()
    }
}

impl<'a> Endpoint for Project<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.statistics
            .map(|value| pairs.append_pair("statistics", common::bool_str(value)));
        self.license
            .map(|value| pairs.append_pair("license", common::bool_str(value)));
        self.with_custom_attributes
            .map(|value| pairs.append_pair("with_custom_attributes", common::bool_str(value)));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::Project;

    #[test]
    fn project_is_necessary() {
        let err = Project::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Project::builder().project(1).build().unwrap();
    }
}
