// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Query for labels within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Labels<'a> {
    /// The project to query for labels.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    /// Include issue and merge request counts..
    #[builder(default)]
    with_counts: Option<bool>,
    /// Include ancestor groups.
    ///
    /// Defaults to `true`.
    #[builder(default)]
    include_ancestor_groups: Option<bool>,
}

impl<'a> Labels<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> LabelsBuilder<'a> {
        LabelsBuilder::default()
    }
}

impl<'a> Endpoint for Labels<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/labels", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.with_counts
            .map(|value| pairs.append_pair("with_counts", common::bool_str(value)));
        self.include_ancestor_groups
            .map(|value| pairs.append_pair("include_ancestor_groups", common::bool_str(value)));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::labels::Labels;

    #[test]
    fn project_is_needed() {
        let err = Labels::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Labels::builder().project(1).build().unwrap();
    }
}
