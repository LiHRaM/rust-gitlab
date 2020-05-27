// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Create a branch on a project.
#[derive(Debug, Builder)]
pub struct CreateBranch<'a> {
    /// The project to create a branch on.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The name of the new branch.
    #[builder(setter(into), default)]
    branch: Cow<'a, str>,
    /// The ref to create the branch from.
    #[builder(setter(into), default)]
    ref_: Cow<'a, str>,
}

impl<'a> CreateBranch<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateBranchBuilder<'a> {
        CreateBranchBuilder::default()
    }
}

impl<'a> Endpoint for CreateBranch<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/repository/branches", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params.push("branch", &self.branch).push("ref", &self.ref_);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::repository::branches::CreateBranch;

    #[test]
    fn project_is_necessary() {
        let err = CreateBranch::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        CreateBranch::builder().project(1).build().unwrap();
    }
}
