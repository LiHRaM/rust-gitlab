// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project pipeline API endpoints.
//!
//! These endpoints are used for querying CI pipelines.

pub mod jobs;

mod cancel;
mod create;
mod delete;
mod pipeline;
mod pipelines;
mod retry;
mod variables;

pub use self::cancel::CancelPipeline;
pub use self::cancel::CancelPipelineBuilder;

pub use self::create::CreatePipeline;
pub use self::create::CreatePipelineBuilder;
pub use self::create::PipelineVariable;
pub use self::create::PipelineVariableType;

pub use self::delete::DeletePipeline;
pub use self::delete::DeletePipelineBuilder;

pub use self::pipeline::Pipeline;
pub use self::pipeline::PipelineBuilder;

pub use self::pipelines::PipelineOrderBy;
pub use self::pipelines::PipelineScope;
pub use self::pipelines::PipelineStatus;
pub use self::pipelines::Pipelines;
pub use self::pipelines::PipelinesBuilder;

pub use self::retry::RetryPipeline;
pub use self::retry::RetryPipelineBuilder;

pub use self::variables::PipelineVariables;
pub use self::variables::PipelineVariablesBuilder;
