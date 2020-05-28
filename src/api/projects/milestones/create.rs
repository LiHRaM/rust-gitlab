// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use chrono::NaiveDate;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Create a new milestone on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateProjectMilestone<'a> {
    /// The project to create a new milestone within.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The title of the milestone.
    #[builder(setter(into))]
    title: Cow<'a, str>,

    /// A short description for the milestone.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,
    /// When the milestone is due.
    #[builder(default)]
    due_date: Option<NaiveDate>,
    /// When the milestone starts.
    #[builder(default)]
    start_date: Option<NaiveDate>,
}

impl<'a> CreateProjectMilestone<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateProjectMilestoneBuilder<'a> {
        CreateProjectMilestoneBuilder::default()
    }
}

impl<'a> Endpoint for CreateProjectMilestone<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/milestones", self.project).into()
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("title", &self.title)
            .push_opt("description", self.description.as_ref())
            .push_opt("due_date", self.due_date)
            .push_opt("start_date", self.start_date);

        params.into_body()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::milestones::CreateProjectMilestone;

    #[test]
    fn project_and_title_are_necessary() {
        let err = CreateProjectMilestone::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateProjectMilestone::builder()
            .title("title")
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn title_is_necessary() {
        let err = CreateProjectMilestone::builder()
            .project("project")
            .build()
            .unwrap_err();
        assert_eq!(err, "`title` must be initialized");
    }

    #[test]
    fn project_and_title_are_sufficient() {
        CreateProjectMilestone::builder()
            .project("project")
            .title("title")
            .build()
            .unwrap();
    }
}
