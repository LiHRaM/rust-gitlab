// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::query_prelude::*;
use crate::types::UserResult;

#[derive(Debug)]
pub struct CurrentUser;

impl<U> SingleQuery<U> for CurrentUser
where
    U: UserResult,
{
    type FormData = ();

    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> String {
        "user".into()
    }

    fn add_parameters(&self, _: Pairs) {}
    fn form_data(&self) {}
}

impl<U> Query<U> for CurrentUser
where
    U: UserResult,
{
    fn query(&self, client: &Gitlab) -> Result<U, GitlabError> {
        self.single_query(client)
    }
}
