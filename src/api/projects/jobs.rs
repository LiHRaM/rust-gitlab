// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;
use std::fmt;

use derive_builder::Builder;

use crate::query_common::NameOrId;
use crate::query_prelude::*;

/// Scopes for jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobScope {
    /// Created, but blocked on dependencies or triggers.
    Created,
    /// Ready to run, but have not been claimed by a runner.
    Pending,
    /// Currently running.
    Running,
    /// Failed jobs.
    Failed,
    /// Successful jobs.
    Success,
    /// Canceled jobs.
    Canceled,
    /// Skipped jobs.
    Skipped,
    /// Awaiting manual triggering.
    Manual,
}

impl JobScope {
    /// The scope as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            JobScope::Created => "created",
            JobScope::Pending => "pending",
            JobScope::Running => "running",
            JobScope::Failed => "failed",
            JobScope::Success => "success",
            JobScope::Canceled => "canceled",
            JobScope::Skipped => "skipped",
            JobScope::Manual => "manual",
        }
    }
}

impl fmt::Display for JobScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Query for jobs within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Jobs<'a> {
    /// The project to query for jobs.
    #[builder(setter(into))]
    project: NameOrId<'a>,

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
        format!("projects/{}/jobs", self.project)
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
    use crate::api::projects::Jobs;

    #[test]
    fn project_is_needed() {
        let err = Jobs::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Jobs::builder().project(1).build().unwrap();
    }
}
