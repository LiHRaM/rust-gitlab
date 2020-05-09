// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::query_common::NameOrId;
use crate::query_prelude::*;

/// Cancel a pipeline.
#[derive(Debug, Builder)]
pub struct CancelPipeline<'a> {
    /// The project to query for the pipeline.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the pipeline.
    pipeline: u64,
}

impl<'a> CancelPipeline<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CancelPipelineBuilder<'a> {
        CancelPipelineBuilder::default()
    }
}

impl<'a, T> SingleQuery<T> for CancelPipeline<'a>
where
    T: DeserializeOwned,
{
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/pipelines/{}/cancel",
            self.project, self.pipeline,
        )
        .into()
    }
}

impl<'a, T> Query<T> for CancelPipeline<'a>
where
    T: DeserializeOwned,
{
    fn query(&self, client: &dyn Client) -> Result<T, GitlabError> {
        self.single_query(client)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::CancelPipeline;

    #[test]
    fn project_and_pipeline_are_needed() {
        let err = CancelPipeline::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = CancelPipeline::builder().pipeline(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn pipeline_is_needed() {
        let err = CancelPipeline::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`pipeline` must be initialized");
    }

    #[test]
    fn project_and_pipeline_are_sufficient() {
        CancelPipeline::builder()
            .project(1)
            .pipeline(1)
            .build()
            .unwrap();
    }
}
