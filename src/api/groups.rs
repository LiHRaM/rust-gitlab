// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(clippy::module_inception)]

//! Group-related API endpoints
//!
//! These endpoints are used for querying and modifying groups and their resources.

mod create;
mod group;
mod groups;

pub use create::BranchProtection;
pub use create::CreateGroup;
pub use create::CreateGroupBuilder;
pub use create::GroupProjectCreationAccessLevel;
pub use create::SubgroupCreationAccessLevel;

pub use group::Group;
pub use group::GroupBuilder;

pub use groups::GroupOrderBy;
pub use groups::Groups;
pub use groups::GroupsBuilder;
