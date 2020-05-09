// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;

use derive_builder::Builder;

use crate::api::projects::JobScope;
use crate::query_common::NameOrId;
use crate::query_prelude::*;

/// Query for jobs within a pipeline.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Jobs<'a> {
    /// The project to query for the pipeline.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the pipeline.
    pipeline: u64,

    /// Pagination to use for the results.
    #[builder(default)]
    pagination: Pagination,

    /// The scopes to filter jobs by.
    #[builder(setter(name = "_scopes"), default, private)]
    scopes: HashSet<JobScope>,
}

impl<'a> Jobs<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> JobsBuilder<'a> {
        JobsBuilder::default()
    }
}

impl<'a> JobsBuilder<'a> {
    /// Filter jobs by a scope.
    pub fn scope(&mut self, scope: JobScope) -> &mut Self {
        self.scopes
            .get_or_insert_with(Default::default)
            .insert(scope);
        self
    }

    /// Filter jobs by a set of scopes.
    pub fn scopes<I>(&mut self, scopes: I) -> &mut Self
    where
        I: Iterator<Item = JobScope>,
    {
        self.scopes
            .get_or_insert_with(Default::default)
            .extend(scopes);
        self
    }
}

impl<'a, T> SingleQuery<Vec<T>> for Jobs<'a>
where
    T: DeserializeOwned,
{
    type FormData = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> String {
        format!("projects/{}/pipelines/{}/jobs", self.project, self.pipeline)
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.scopes.iter().for_each(|value| {
            pairs.append_pair("scope[]", value.as_str());
        });
    }

    fn form_data(&self) {}
}

impl<'a, T> PagedQuery<T, ()> for Jobs<'a>
where
    T: DeserializeOwned,
{
    fn pagination(&self) -> Pagination {
        self.pagination
    }
}

impl<'a, T> Query<Vec<T>> for Jobs<'a>
where
    T: DeserializeOwned,
{
    fn query(&self, client: &Gitlab) -> Result<Vec<T>, GitlabError> {
        self.paged_query(client)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::pipelines::jobs::Jobs;

    #[test]
    fn project_and_pipeline_are_needed() {
        let err = Jobs::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_needed() {
        let err = Jobs::builder().pipeline(1).build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn pipeline_is_needed() {
        let err = Jobs::builder().project(1).build().unwrap_err();
        assert_eq!(err, "`pipeline` must be initialized");
    }

    #[test]
    fn project_and_pipeline_are_sufficient() {
        Jobs::builder().project(1).pipeline(1).build().unwrap();
    }
}
