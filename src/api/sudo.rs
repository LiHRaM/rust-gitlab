// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::api::endpoint_prelude::*;

/// A `sudo` modifier that can be applied to any endpoint.
#[derive(Debug, Clone)]
pub struct SudoContext<'a> {
    /// The username to use for the endpoint.
    sudo: Cow<'a, str>,
}

impl<'a> SudoContext<'a> {
    /// Create a new `sudo` context for API endpoints.
    pub fn new<S>(sudo: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        SudoContext {
            sudo: sudo.into(),
        }
    }

    /// Apply the context to an endpoint.
    pub fn apply<E>(&self, endpoint: E) -> Sudo<'a, E> {
        Sudo {
            endpoint,
            sudo: self.sudo.clone(),
        }
    }
}

/// Query information about the API calling user.
#[derive(Debug, Clone)]
pub struct Sudo<'a, E> {
    /// The endpoint to call with `sudo`.
    endpoint: E,

    /// The username to use for the endpoint.
    sudo: Cow<'a, str>,
}

/// Create a `sudo`-elevated version of an endpoint.
pub fn sudo<'a, E, S>(endpoint: E, sudo: S) -> Sudo<'a, E>
where
    S: Into<Cow<'a, str>>,
{
    Sudo {
        endpoint,
        sudo: sudo.into(),
    }
}

impl<'a, E> Endpoint for Sudo<'a, E>
where
    E: Endpoint,
{
    fn method(&self) -> Method {
        self.endpoint.method()
    }

    fn endpoint(&self) -> Cow<'static, str> {
        self.endpoint.endpoint()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = self.endpoint.parameters();
        params.push("sudo", &self.sudo);
        params
    }

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        self.endpoint.body()
    }
}

impl<'a, E> Pageable for Sudo<'a, E>
where
    E: Pageable,
{
    fn use_keyset_pagination(&self) -> bool {
        self.endpoint.use_keyset_pagination()
    }
}
