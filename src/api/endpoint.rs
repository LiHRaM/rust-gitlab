// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use reqwest::{header, Method};
use serde::de::DeserializeOwned;
use url::form_urlencoded::Serializer;
use url::UrlQuery;

use crate::api::{ApiError, BodyError, Client, Query};

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

    /// The body for the endpoint.
    ///
    /// Returns the `Content-Encoding` header for the data as well as the data itself.
    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        Ok(None)
    }
}

impl<E, T, C> Query<T, C> for E
where
    E: Endpoint,
    T: DeserializeOwned,
    C: Client,
{
    fn query(&self, client: &C) -> Result<T, ApiError<C::Error>> {
        let mut url = client.rest_endpoint(&self.endpoint())?;
        self.add_parameters(url.query_pairs_mut());

        let req = client.build_rest(self.method(), url);
        let req = if let Some((mime, data)) = self.body()? {
            req.header(header::CONTENT_TYPE, mime).body(data)
        } else {
            req
        };
        let rsp = client.rest(req)?;
        let status = rsp.status();
        let v = serde_json::from_reader(rsp)?;
        if !status.is_success() {
            return Err(ApiError::from_gitlab(v));
        }

        serde_json::from_value::<T>(v).map_err(ApiError::data_type::<T>)
    }
}
