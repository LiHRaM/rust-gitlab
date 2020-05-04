// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::query_prelude::*;

/// Query a user by ID.
#[derive(Debug, Clone, Copy, Builder)]
pub struct User {
    /// The ID of the user.
    user: u64,
}

impl User {
    /// Create a builder for the endpoint.
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
}

impl<T> SingleQuery<T> for User
where
    T: DeserializeOwned,
{
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("users/{}", self.user).into()
    }
}

impl<T> Query<T> for User
where
    T: DeserializeOwned,
{
    fn query(&self, client: &dyn GitlabClient) -> Result<T, GitlabError> {
        self.single_query(client)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::users::User;

    #[test]
    fn user_is_needed() {
        let err = User::builder().build().unwrap_err();
        assert_eq!(err, "`user` must be initialized");
    }

    #[test]
    fn user_is_sufficient() {
        User::builder().user(1).build().unwrap();
    }
}
