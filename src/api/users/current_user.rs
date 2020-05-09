// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::query_prelude::*;

/// Query information about the API calling user.
#[derive(Debug, Clone, Copy, Builder)]
pub struct CurrentUser {}

impl CurrentUser {
    /// Create a builder for the endpoint.
    pub fn builder() -> CurrentUserBuilder {
        CurrentUserBuilder::default()
    }
}

impl<T> SingleQuery<T> for CurrentUser
where
    T: DeserializeOwned,
{
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "user".into()
    }
}

impl<T> Query<T> for CurrentUser
where
    T: DeserializeOwned,
{
    fn query(&self, client: &dyn Client) -> Result<T, GitlabError> {
        self.single_query(client)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::users::CurrentUser;

    #[test]
    fn defaults_are_sufficient() {
        CurrentUser::builder().build().unwrap();
    }
}
