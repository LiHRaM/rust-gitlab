// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use reqwest::Method;
use serde::de::DeserializeOwned;
use url::form_urlencoded::Serializer;
use url::UrlQuery;

use crate::api::{Client, Query};
use crate::gitlab::GitlabError;

/// A type for managing query parameters.
pub type Pairs<'a> = Serializer<'a, UrlQuery<'a>>;

/// A trait for providing the necessary information for a single REST API endpoint.
pub trait Endpoint {
    /// The HTTP method to use for the endpoint.
    fn method(&self) -> Method;
    /// The path to the endpoint.
    fn endpoint(&self) -> Cow<'static, str>;

    /// Add query parameters for the endpoint.
    #[allow(unused_variables)]
    fn add_parameters(&self, pairs: Pairs) {}

    /// Form data for the endpoint.
    fn form_data(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl<E, T> Query<T> for E
where
    E: Endpoint,
    T: DeserializeOwned,
{
    fn query(&self, client: &dyn Client) -> Result<T, GitlabError> {
        let mut url = client.rest_endpoint(&self.endpoint())?;
        self.add_parameters(url.query_pairs_mut());

        let req = client
            .build_rest(self.method(), url)
            .form(&self.form_data());
        let rsp = client.rest(req)?;
        let status = rsp.status();
        let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
        if !status.is_success() {
            return Err(GitlabError::from_gitlab(v));
        }

        serde_json::from_value::<T>(v).map_err(GitlabError::data_type::<T>)
    }
}
