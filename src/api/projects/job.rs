// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::query_common::NameOrId;
use crate::query_prelude::*;

/// Query for a job within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Job<'a> {
    /// The project to query for the job.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the job.
    job: u64,
}

impl<'a> Job<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> JobBuilder<'a> {
        JobBuilder::default()
    }
}

impl<'a, T> SingleQuery<T> for Job<'a>
where
    T: DeserializeOwned,
{
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/jobs/{}", self.project, self.job).into()
    }
}

impl<'a, T> Query<T> for Job<'a>
where
    T: DeserializeOwned,
{
    fn query(&self, client: &dyn Client) -> Result<T, GitlabError> {
        self.single_query(client)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::Pipelines;

    #[test]
    fn project_is_needed() {
        let err = Pipelines::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Pipelines::builder().project(1).build().unwrap();
    }
}
