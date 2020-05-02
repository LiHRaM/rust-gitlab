// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::query_prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct User {
    pub id: u64,
}

impl<T> SingleQuery<T> for User
where
    T: DeserializeOwned,
{
    type FormData = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> String {
        format!("users/{}", self.id)
    }

    fn add_parameters(&self, _: Pairs) {}
    fn form_data(&self) {}
}

impl<T> Query<T> for User
where
    T: DeserializeOwned,
{
    fn query(&self, client: &Gitlab) -> Result<T, GitlabError> {
        self.single_query(client)
    }
}
