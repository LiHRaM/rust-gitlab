// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(clippy::module_inception)]

//! Project-related API endpoints
//!
//! These endpoints are used for querying and modifying projects and their resources.

mod create;
mod job;
mod jobs;
pub mod pipelines;
mod projects;

pub use self::create::AutoDevOpsDeployStrategy;
pub use self::create::BuildGitStrategy;
pub use self::create::ContainerExpirationCadence;
pub use self::create::ContainerExpirationKeepN;
pub use self::create::ContainerExpirationOlderThan;
pub use self::create::ContainerExpirationPolicy;
pub use self::create::ContainerExpirationPolicyBuilder;
pub use self::create::CreateProject;
pub use self::create::CreateProjectBuilder;
pub use self::create::FeatureAccessLevel;
pub use self::create::FeatureAccessLevelPublic;
pub use self::create::MergeMethod;

pub use self::job::Job;
pub use self::job::JobBuilder;

pub use self::jobs::JobScope;
pub use self::jobs::Jobs;
pub use self::jobs::JobsBuilder;

pub use self::projects::ProjectOrderBy;
pub use self::projects::Projects;
pub use self::projects::ProjectsBuilder;
