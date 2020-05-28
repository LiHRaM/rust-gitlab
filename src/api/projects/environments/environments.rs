// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
use crate::api::endpoint_prelude::*;

#[derive(Debug, Clone)]
enum NameOrSearch<'a> {
    Name(Cow<'a, str>),
    Search(Cow<'a, str>),
}

/// Query for environments within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Environments<'a> {
    /// The project to query for environments.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    #[builder(setter(name = "_name_or_search"), default, private)]
    name_or_search: Option<NameOrSearch<'a>>,
}

impl<'a> Environments<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EnvironmentsBuilder<'a> {
        EnvironmentsBuilder::default()
    }
}

impl<'a> EnvironmentsBuilder<'a> {
    /// Filter environments matching a name.
    ///
    /// Mutually exclusive with `search`.
    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: Into<Cow<'a, str>>,
    {
        self.name_or_search = Some(Some(NameOrSearch::Name(name.into())));
        self
    }

    /// Filter environments matching a search criteria.
    ///
    /// Mutually exclusive with `name`.
    pub fn search<S>(&mut self, search: S) -> &mut Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.name_or_search = Some(Some(NameOrSearch::Search(search.into())));
        self
    }
}

impl<'a> Endpoint for Environments<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/environments", self.project).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        if let Some(name_or_search) = self.name_or_search.as_ref() {
            match name_or_search {
                NameOrSearch::Name(name) => {
                    params.push("name", name);
                },
                NameOrSearch::Search(search) => {
                    params.push("search", search);
                },
            }
        }

        params
    }
}

impl<'a> Pageable for Environments<'a> {}

#[cfg(test)]
mod tests {
    use crate::api::projects::environments::Environments;

    #[test]
    fn project_is_needed() {
        let err = Environments::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        Environments::builder().project(1).build().unwrap();
    }
}
