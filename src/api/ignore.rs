// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::api::{Client, Endpoint, Query};
use crate::gitlab::GitlabError;

/// A query modifier that ignores the data returned from an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ignore<E> {
    endpoint: E,
}

/// Ignore the resulting data from an endpoint.
pub fn ignore<E>(endpoint: E) -> Ignore<E> {
    Ignore {
        endpoint,
    }
}

impl<E> Query<()> for Ignore<E>
where
    E: Endpoint,
{
    fn query(&self, client: &dyn Client) -> Result<(), GitlabError> {
        let mut url = client.rest_endpoint(&self.endpoint.endpoint())?;
        self.endpoint.add_parameters(url.query_pairs_mut());

        let req = client
            .build_rest(self.endpoint.method(), url)
            .form(&self.endpoint.form_data());
        let rsp = client.rest(req)?;
        if !rsp.status().is_success() {
            let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
            return Err(GitlabError::from_gitlab(v));
        }

        Ok(())
    }
}
