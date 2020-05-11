// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for a specific branch in a project.
#[derive(Debug, Builder)]
pub struct Branches<'a> {
    /// The project to get a branch from.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// Filter branches by a search query.
    #[builder(setter(into), default)]
    search: Option<Cow<'a, str>>,
}

impl<'a> Branches<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> BranchesBuilder<'a> {
        BranchesBuilder::default()
    }
}

impl<'a> Endpoint for Branches<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/repository/branches", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.search
            .as_ref()
            .map(|value| pairs.append_pair("search", value));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::repository::branches::Branches;

    #[test]
    fn project_is_necessary() {
        let err = Branches::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Branches::builder().project(1).build().unwrap();
    }
}
