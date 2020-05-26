// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Create a label within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateLabel<'a> {
    /// The project to create a label within.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The name of the label.
    #[builder(setter(into))]
    name: Cow<'a, str>,
    /// The color of the label.
    ///
    /// CSS and RGB colors in `#RRGGBB` format are supported.
    ///
    /// TODO: Use a specific structure for this.
    #[builder(setter(into))]
    color: Cow<'a, str>,

    /// The description of the label.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// The priority of the label.
    #[builder(default)]
    priority: Option<u64>,
}

impl<'a> CreateLabel<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateLabelBuilder<'a> {
        CreateLabelBuilder::default()
    }
}

impl<'a> Endpoint for CreateLabel<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/labels", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("name", &self.name)
            .push("color", &self.color)
            .push_opt("description", self.description.as_ref())
            .push_opt("priority", self.priority);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::labels::CreateLabel;

    #[test]
    fn project_name_and_color_are_necessary() {
        let err = CreateLabel::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateLabel::builder()
            .name("label")
            .color("#f100fe")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn name_is_necessary() {
        let err = CreateLabel::builder()
            .project(1)
            .color("#f100fe")
            .build()
            .unwrap_err();
        assert_eq!(err, "`name` must be initialized");
    }

    #[test]
    fn color_is_necessary() {
        let err = CreateLabel::builder()
            .project(1)
            .name("label")
            .build()
            .unwrap_err();
        assert_eq!(err, "`color` must be initialized");
    }

    #[test]
    fn project_name_and_color_are_sufficient() {
        CreateLabel::builder()
            .project(1)
            .name("label")
            .color("#f100fe")
            .build()
            .unwrap();
    }
}
