// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use reqwest::blocking::{RequestBuilder, Response};
use reqwest::Method;
use serde::de::DeserializeOwned;
use url::form_urlencoded::Serializer;
use url::{Url, UrlQuery};

use crate::gitlab::GitlabError;

pub type Pairs<'a> = Serializer<'a, UrlQuery<'a>>;

pub trait GitlabClient {
    /// Get the URL for the endpoint for the client.
    ///
    /// This method adds the hostname for the client's target instance.
    fn rest_endpoint(&self, endpoint: &str) -> Result<Url, GitlabError>;

    /// Build a REST query from a URL and a given method.
    fn build_rest(&self, method: Method, url: Url) -> RequestBuilder;

    /// Send a REST query.
    fn rest(&self, request: RequestBuilder) -> Result<Response, GitlabError>;
}

pub trait Query<T> {
    fn query(&self, client: &dyn GitlabClient) -> Result<T, GitlabError>;
}

pub trait SingleQuery<T>
where
    T: DeserializeOwned,
{
    fn method(&self) -> Method;
    fn endpoint(&self) -> Cow<'static, str>;

    #[allow(unused_variables)]
    fn add_parameters(&self, pairs: Pairs) {}

    fn form_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn single_query(&self, client: &dyn GitlabClient) -> Result<T, GitlabError> {
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

    fn no_answer_query(&self, client: &dyn GitlabClient) -> Result<(), GitlabError> {
        let mut url = client.rest_endpoint(&self.endpoint())?;
        self.add_parameters(url.query_pairs_mut());

        let req = client
            .build_rest(self.method(), url)
            .form(&self.form_data());
        let rsp = client.rest(req)?;
        if !rsp.status().is_success() {
            let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
            return Err(GitlabError::from_gitlab(v));
        }

        Ok(())
    }
}
