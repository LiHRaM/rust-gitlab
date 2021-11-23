// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(clippy::module_inception)]

//! User-related API endpoints
//!
//! These endpoints are used for querying and modifying users and their resources.

mod current_user;
mod user;
mod users;

pub use self::user::User;
pub use self::user::UserBuilder;

pub use self::current_user::CurrentUser;
pub use self::current_user::CurrentUserBuilder;

pub use self::users::ExternalProvider;
pub use self::users::ExternalProviderBuilder;
pub use self::users::UserOrderBy;
pub use self::users::Users;
pub use self::users::UsersBuilder;
