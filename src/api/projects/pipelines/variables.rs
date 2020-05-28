// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Query for the variables of a pipeline.
#[derive(Debug, Builder)]
pub struct PipelineVariables<'a> {
    /// The project of the pipelines.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the pipeline.
    pipeline: u64,
}

impl<'a> PipelineVariables<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> PipelineVariablesBuilder<'a> {
        PipelineVariablesBuilder::default()
    }
}

impl<'a> Endpoint for PipelineVariables<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/pipelines/{}/variables",
            self.project, self.pipeline,
        )
        .into()
    }
}

impl<'a> Pageable for PipelineVariables<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::PipelineVariables;

    #[test]
    fn project_and_pipeline_are_needed() {
        let err = PipelineVariables::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = PipelineVariables::builder()
            .pipeline(1)
            .build()
            .unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn pipeline_is_needed() {
        let err = PipelineVariables::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`pipeline` must be initialized");
    }

    #[test]
    fn project_and_pipeline_are_sufficient() {
        PipelineVariables::builder()
            .project(1)
            .pipeline(1)
            .build()
            .unwrap();
    }
}
