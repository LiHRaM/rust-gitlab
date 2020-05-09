// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::endpoint_prelude::*;
use crate::query_common::NameOrId;

/// Query a single pipeline on a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Pipeline<'a> {
    /// The project to query for pipeline.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the pipeline.
    pipeline: u64,
}

impl<'a> Pipeline<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> PipelineBuilder<'a> {
        PipelineBuilder::default()
    }
}

impl<'a> Endpoint for Pipeline<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/pipeline/{}", self.project, self.pipeline).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::Pipeline;

    #[test]
    fn project_and_pipeline_are_needed() {
        let err = Pipeline::builder().pipeline(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = Pipeline::builder().pipeline(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn pipeline_is_needed() {
        let err = Pipeline::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`pipeline` must be initialized");
    }

    #[test]
    fn project_and_pipeline_are_sufficient() {
        Pipeline::builder().project(1).pipeline(1).build().unwrap();
    }
}
