// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Query for a label within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Label<'a> {
    /// The project to query for the label.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the label.
    label: u64,

    /// Include ancestor groups.
    ///
    /// Defaults to `true`.
    #[builder(default)]
    include_ancestor_groups: Option<bool>,
}

impl<'a> Label<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> LabelBuilder<'a> {
        LabelBuilder::default()
    }
}

impl<'a> Endpoint for Label<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/labels/{}", self.project, self.label).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.include_ancestor_groups
            .map(|value| pairs.append_pair("include_ancestor_groups", common::bool_str(value)));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::labels::Label;

    #[test]
    fn project_and_label_are_needed() {
        let err = Label::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = Label::builder().label(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn label_is_needed() {
        let err = Label::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`label` must be initialized");
    }

    #[test]
    fn project_and_label_are_sufficient() {
        Label::builder().project(1).label(1).build().unwrap();
    }
}
