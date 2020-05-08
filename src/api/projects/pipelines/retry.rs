// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::query_common::NameOrId;
use crate::query_prelude::*;

/// Retry a pipeline.
///
/// Note that this only restarts failed jobs.
#[derive(Debug, Builder)]
pub struct RetryPipeline<'a> {
    /// The project of the pipelines.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the pipeline.
    pipeline: u64,
}

impl<'a> RetryPipeline<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> RetryPipelineBuilder<'a> {
        RetryPipelineBuilder::default()
    }
}

impl<'a, T> SingleQuery<T> for RetryPipeline<'a>
where
    T: DeserializeOwned,
{
    type FormData = ();

    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> String {
        format!(
            "projects/{}/pipelines/{}/retry",
            self.project, self.pipeline,
        )
    }

    fn add_parameters(&self, _: Pairs) {}
    fn form_data(&self) {}
}

impl<'a, T> Query<T> for RetryPipeline<'a>
where
    T: DeserializeOwned,
{
    fn query(&self, client: &Gitlab) -> Result<T, GitlabError> {
        self.single_query(client)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::RetryPipeline;

    #[test]
    fn project_and_pipeline_are_needed() {
        let err = RetryPipeline::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = RetryPipeline::builder().pipeline(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn pipeline_is_needed() {
        let err = RetryPipeline::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`pipeline` must be initialized");
    }

    #[test]
    fn project_and_pipeline_are_sufficient() {
        RetryPipeline::builder()
            .project(1)
            .pipeline(1)
            .build()
            .unwrap();
    }
}
