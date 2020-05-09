// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

/// Delete a pipeline.
#[derive(Debug, Builder)]
pub struct DeletePipeline<'a> {
    /// The project to delete the pipeline from.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the pipeline.
    pipeline: u64,
}

impl<'a> DeletePipeline<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> DeletePipelineBuilder<'a> {
        DeletePipelineBuilder::default()
    }
}

impl<'a> Endpoint for DeletePipeline<'a> {
    fn method(&self) -> Method {
        Method::DELETE
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/pipelines/{}", self.project, self.pipeline).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::DeletePipeline;

    #[test]
    fn project_and_pipeline_are_needed() {
        let err = DeletePipeline::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = DeletePipeline::builder().pipeline(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn pipeline_is_needed() {
        let err = DeletePipeline::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`pipeline` must be initialized");
    }

    #[test]
    fn project_and_pipeline_are_sufficient() {
        DeletePipeline::builder()
            .project(1)
            .pipeline(1)
            .build()
            .unwrap();
    }
}
