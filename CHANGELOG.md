# v0.1300.0 (unreleased)

## Changes

  * The `api::projects::issues::Issues` endpoint's `milestone` field was
    changed to match the actual API exposed by GitLab (with `None` and `Any`
    options).
  * The `api::projects::pipelines::PipelineVariables` endpoint is now pageable.

# v0.1210.2

## New request body handling

It was observed (#41) that the new API pattern was not handling `POST` and
`PUT` parameters properly. This has now been fixed.

## New request parameter handling

In the process of updating the body handling, a simpler pattern for query
parameters was also implemented.

## Additional merge status cases

Some additional merge status names for merge requests were missing and have
been added.

## Fixes

  * The `api::projects::environments::Environment` endpoint uses the correct
    path now.
  * The `api::groups::members::GroupMembers`,
    `api::projects::members::ProjectMembers`, and
    `api::projects::repository::Branches` endpoints now accepts plain strings
    for their `query` fields.
  * The `api::projects::protected_branches::UnprotectBranch` endpoint now
    properly escapes branch names with URL-special characters.
  * The `api::projects::repository::CreateFile` endpoint now properly upgrades
    the encoding when attempting to encode binary contents using
    `Encoding::Text`.
  * The `api::projects::CreateProject` and `api::projects::EditProject`
    endpoints now accepts plain strings in its `import_url` field.

## Changes

  * The `api::projects::issues::EditIssue` now uses `issue` rather than
    `issue_iid` for consistency.

# v0.1210.1

## New API strategy

A new pattern for API implementation is now underway. Instead of methods
directly on the `Gitlab` instance, there are now structures which implement an
`api::Endpoint` trait. This trait may be used to query any structure
implementing the `api::Client` trait using the `api::Query` trait. All
endpoints use the "builder" pattern to collect required and optional
parameters.

There are some adaptor functions to handle various use cases:

  - `api::paged`: This may be used to handle pagination of any endpoint which
    supports it (checked at compile time).
  - `api::ignore`: This may be used to ignore the content of the response for
    any endpoint. HTTP and GitLab error messages are still captured.
  - `api::raw`: Instead of deserializing the contents of the result from GitLab
    into a structure, the raw bytes may be fetched instead using this function.
  - `api::sudo`: This function adapts any endpoint into being called as another
    user if the client is able to do so (basically, is an administrator).

The `api::Query` trait deserializes the contents from GitLab into any structure
which implements the `serde::DeserializeOwned` trait. This can be used to only
grab information of interest to the caller instead of extracting all of the
information available through the `types` module.

If your endpoint is deprecated, it has been marked as such and you should
migrate to the new pattern. Please see the docs for available endpoints.

All new endpoint implementations should use the new pattern rather than adding
methods to `Gitlab`. Result structures do not need to be added to this crate
either. It is expected that they too will be deprecated at some point and
either not provided or moved to a dedicated crate.

### Examples:

```rust
use std::env;

use serde::Deserialize;
use gitlab::Gitlab;
use gitlab::api::{self, projects, Query};

#[derive(Debug, Deserialize)]
struct Project {
    name: String,
}

fn example() {
    // Create the client.
    let client = Gitlab::new("gitlab.com", env::get("GITLAB_TOKEN").unwrap()).unwrap();

    // Create a simple endpoint.
    let endpoint = projects::Project::builder().project("gitlab-org/gitlab").build().unwrap();
    // Get the information.
    let project: Project = endpoint.query(&client).unwrap();
    // Call it again, but ignore the response from GitLab.
    let _: () = api::ignore(endpoint).query(&client).unwrap();

    // Create an endpoint that supports pagination.
    let pageable_endpoint = projects::Projects::builder().build().unwrap();
    // Get just the first page (20 results).
    let first_page: Vec<Project> = pageable_endpoint.query(&client).unwrap();
    // Get 200 results instead.
    let first_200_projects: Vec<Project> = api::paged(pageable_endpoint, api::Pagination::Limit(200)).query(&client).unwrap();

    // Query `gitlab-org/gitlab` except by ID this time.
    let endpoint = projects::Project::builder().project(278964).build().unwrap();
    // Get the raw data from the response.
    let raw_data: Vec<u8> = api::raw(endpoint).query(&client).unwrap();
}
```

## Changes

  * Include a changelog.
