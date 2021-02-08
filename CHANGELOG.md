# v0.1308.1 (unreleased)

## Breaking changes

  * `ParamValue::as_value` now takes its value as `&self` rather than `self`.
    This was required in order to implement `CommaSeparatedList` reliably.

## Additions

  * `api::common::CommaSeparatedList` now exists for easy use of
    comma-separated values.

## Deprecations

  * `EditIssue::remove_labels` is deprecated in favor of the better
    `clear_labels` wording.

## Changes

  * API bindings for the `"minimal"` access level.
  * Groups can have "inherit" set as their shared runner minute limit.
  * Listing groups can now be set to only return top-level groups.
  * Searching for projects within a group can now be sorted by a similarity
    score based on the search criteria.
  * Project container expiration policies can now use an arbitrary "keep n"
    count.
  * Project container expiration policies now have `name_regex_delete`
    (replacing the now-deprecated `name_regex`) and `name_regex_keep`.
  * Projects can now be created and edited with `operations_access_level`
    settings.
  * Projects can now be created and edited with `requirements_access_level`
    settings.
  * Projects can now be created and edited with `analytics_access_level`
    settings.
  * Projects can now be created and edited with `show_default_award_emojis`
    settings.
  * Projects can now be created and edited with
    `restrict_user_defined_variables` settings.
  * Projects can now be created and edited with
    `allow_merge_on_skipped_pipeline` settings.
  * Projects can now be edited with `ci_forward_deployment_enabled` settings.
  * Environments can now be filtered by their deployment state.
  * Project hooks can now be registered for events related to confidential
    notes, deployments, and releases.
  * Issues can now be edited with incremental label changes.
  * Issues can now be filtered by iterations, due dates, and search queries can
    now be scoped.
  * Issue notes can now be created and edited with the confidential flag.
  * Project labels can be filtered by search queries.
  * Project members can now be edited in batch (using multiple IDs).

# v0.1308.0

  * No changes needed for GitLab 13.8.

# v0.1307.0

## Additions

  * Added `api::projects::repository::files::FileRaw`
  * Added `api::projects::merge_requests::approval_state::MergeRequestApprovalState`
    query to access the approval rules state of a particular merge request.

# v0.1306.0

  * No changes needed for GitLab 13.6.

# v0.1305.1

## Changes

  * Changed `ci_config_path` to `Option<String>` in `gitlab::webhooks::PipelineHookAttrs`

# v0.1305.0

## Additions

  * Added `head_pipeline_id` field to `gitlab::webhooks::MergeRequestHookAttrs`

# v0.1304.0

## Changes

  * Error types now use `#[non_exhaustive]`

# v0.1303.0

## Additions

  * Added `gitlab::webhooks::PipelineHook`

# v0.1302.2

## Additions

  * `Id` types now implement `Hash`

# v0.1302.1

## Additions

  * Added `api::projects::issues::MergeRequestsClosing` and
    `api::projects::issues::MergeRequestsClosing`

## Fixes

  * GitLab 13.2 added the `approved` and `unapproved` merge request actions for
    CE.

# v0.1302.0

## Additions

  * Added the `api::projects::protected_tags::ProtectTag`
    `api::projects::protected_tags::UnprotectTag`
    `api::projects::protected_tags::ProtectedTag`
    `api::projects::protected_tags::ProtectedTags` endpoint to query, protect
    and unprotect a projects tags.
  * Added the `api::projects::labels::DeleteLabel` endpoint to delete existing
    labels from a project.
  * Added the `api::projects::labels::PromoteLabel` endpoint to promote a project
    label to a group label.
  * Added the `api::projects:merge_requests::MergeMergeRequest` endpoint to
    merge open merge requests.
  * Added the `api::projects:merge_requests::RebaseMergeRequest` endpoint to
    rebase open merge requests when using the fast-forward merge model.
  * Added the `api::projects:merge_requests::ApproveMergeRequest` endpoint to
    approve open merge requests.
  * Added the `api::projects:merge_requests::UnapproveMergeRequest` endpoint to
    unapprove approved merge requests.

# v0.1301.1

## Changes

  * Updated `api::projects::members::ProjectMember[s]` to support the ability
    to include member details for those members that have access as a result
    of belonging to ancestor/enclosing groups, in addition to directly added
    members.
  * Allow a label via the `api::projects::labels::Label` endpoint to be queried
    by id or name.

## Additions

  * Added the `api::groups::projects::GroupProjects` endpoint to list a groups
    projects.
  * Added the `api::groups::subgroups::GroupSubgroups` endpoint to list a
    groups subgroups.
  * Added the `api::projects::protected_branches::ProtectedBranches` endpoint
    to list a projects protected branches.
  * Added the `api::projects::protected_branches::ProtectedBranch` endpoint
    to query a projects protected branch.

## Fixes

  * Added pagination support to `api::projects::labels::Labels`
  * Keyset pagination also supports the to-be-removed (14.0) `Links` HTTP
    header.

# v0.1301.0

## Deprecations

  * The REST endpoint methods on the `Gitlab` structure have been removed.
    Associated helper structures for resource creation endpoints have been
    removed as well:
    - `CreateMergeRequestParams`
    - `CreateMergeRequestParamsBuilder`
    - `CreateGroupParams`
    - `CreateGroupParamsBuilder`
    - `CreateProjectParams`
    - `CreateProjectParamsBuilder`
    - `MergeMethod`
    - `BuildGitStrategy`
    - `AutoDeployStrategy`
    - `WebhookEvents`
    - `CommitStatusInfo`
    - `MergeRequestStateFilter`
    - `RepoFile`
    - `ProjectFeatures`
    - `QueryParamSlice`
    - `QueryParamVec`
  * Now-impossible error conditions have been removed from `GitlabError`.

# v0.1300.0

## Deprecations

  * All methods on the `Gitlab` structure now have `Endpoint` structures
    implemented. In a future release, these methods (and their support types)
    will be removed.
  * The `Serialize` implementations of the API types are deprecated (though
    marking them as such is difficult).

## Changes

  * The `api::projects::issues::Issues` endpoint's `milestone` field was
    changed to match the actual API exposed by GitLab (with `None` and `Any`
    options).
  * The `api::projects::pipelines::PipelineVariables` endpoint is now pageable.
  * All `EnableState` fields may now be set using `bool` values.
  * The `api::projects::merge_requests::EditMergeRequest` endpoint now supports
    unlabeling a merge request.
  * The `api::Client` trait has been changed to use the `http` crate types.
    This allows for clients to not be tied to `reqwest` and for mocking and
    testing of the endpoints themselves.
  * GitLab errors now detect error objects returned from the API.

## Fixes

  * The `min_access_level` field for `api::groups::Groups` and the
    `access_level` for `api::projects::members::AddProjectMember` are now
    properly passed as integers to the API. (#42)
  * The path used for the project and group milestone endpoints has been fixed.

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
