// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::collections::HashSet;

use derive_builder::Builder;

use crate::api::common::{self, EnableState, NameOrId, VisibilityLevel};
use crate::api::endpoint_prelude::*;
use crate::api::projects::{
    AutoDevOpsDeployStrategy, BuildGitStrategy, ContainerExpirationPolicy, FeatureAccessLevel,
    FeatureAccessLevelPublic, MergeMethod,
};

/// Edit an existing project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct EditProject<'a> {
    /// The project to edit.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    /// The name of the project.
    #[builder(setter(into), default)]
    name: Option<Cow<'a, str>>,
    /// The path of the project.
    #[builder(setter(into), default)]
    path: Option<Cow<'a, str>>,

    /// The default branch of the new project.
    #[builder(setter(into), default)]
    default_branch: Option<Cow<'a, str>>,
    /// The description of the new project.
    #[builder(setter(into), default)]
    description: Option<Cow<'a, str>>,

    /// Set the access level for issues.
    #[builder(default)]
    issues_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for repository access.
    #[builder(default)]
    repository_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for merge requests.
    #[builder(default)]
    merge_requests_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for making a fork of the project.
    #[builder(default)]
    forking_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for CI pipeline access.
    #[builder(default)]
    builds_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for access to view the wiki.
    #[builder(default)]
    wiki_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for snippets.
    #[builder(default)]
    snippets_access_level: Option<FeatureAccessLevel>,
    /// Set the access level for GitLab Pages on the project.
    #[builder(default)]
    pages_access_level: Option<FeatureAccessLevelPublic>,

    /// Whether to enable email notifications or not.
    #[builder(default)]
    emails_disabled: Option<bool>,
    /// Whether outdated diff discussions are resolved when a merge request is updated or not.
    #[builder(default)]
    resolve_outdated_diff_discussions: Option<bool>,
    /// Whether the container registry is enabled or not.
    #[builder(default)]
    container_registry_enabled: Option<bool>,
    /// The expiration policy for containers.
    #[builder(default)]
    container_expiration_policy_attributes: Option<ContainerExpirationPolicy<'a>>,
    /// Whether the project can use shared runners or not.
    #[builder(default)]
    shared_runners_enabled: Option<bool>,
    /// The visibility level of the project.
    #[builder(default)]
    visibility: Option<VisibilityLevel>,
    /// A URL to import the repository from.
    #[builder(default)]
    import_url: Option<Cow<'a, str>>,
    /// Whether job results are visible to non-project members or not.
    #[builder(default)]
    public_builds: Option<bool>,
    /// Whether the CI pipeline is required to succeed before merges are allowed.
    #[builder(default)]
    only_allow_merge_if_pipeline_succeeds: Option<bool>,
    /// Whether all discussions must be resolved before merges are allowed.
    #[builder(default)]
    only_allow_merge_if_all_discussions_are_resolved: Option<bool>,
    /// The merge method to use for the project.
    #[builder(default)]
    merge_method: Option<MergeMethod>,
    /// Whether issues referenced on the default branch should be closed or not.
    #[builder(default)]
    autoclose_referenced_issues: Option<bool>,
    /// The commit message to use for code suggestion commits.
    #[builder(setter(into), default)]
    suggestion_commit_message: Option<Cow<'a, str>>,
    /// Whether to enabled the "Remove source branch" option in new merge requests by default or
    /// not.
    #[builder(default)]
    remove_source_branch_after_merge: Option<bool>,
    /// Whether `git-lfs` support should be enabled or not.
    ///
    /// See the [git-lfs](https://git-lfs.github.com/) website for more information.
    #[builder(default)]
    lfs_enabled: Option<bool>,
    /// Whether users may request access to the repository or not.
    #[builder(default)]
    request_access_enabled: Option<bool>,
    /// A list of tags to apply to the repository.
    #[builder(setter(name = "_tag_list"), default, private)]
    tag_list: HashSet<Cow<'a, str>>,
    // TODO: Figure out how to actuall use this.
    // avatar   mixed   no  Image file for avatar of the project
    // avatar: ???,
    /// The default Git strategy for CI jobs of the project.
    #[builder(default)]
    build_git_strategy: Option<BuildGitStrategy>,
    /// The default timeout for jobs of the project (in seconds).
    #[builder(default)]
    build_timeout: Option<u64>,
    /// Whether to automatically cancel pipelines when branches are updated when using a previous
    /// version of th branch.
    #[builder(default)]
    auto_cancel_pending_pipelines: Option<EnableState>,
    /// The default regular expression to use for build coverage extraction.
    #[builder(setter(into), default)]
    build_coverage_regex: Option<Cow<'a, str>>,
    /// The path to the GitLab CI configuration file within the repository.
    ///
    /// Defaults to `.gitlab-ci.yml`.
    #[builder(setter(into), default)]
    ci_config_path: Option<Cow<'a, str>>,
    /// The default number of revisions to fetch in CI jobs.
    #[builder(default)]
    ci_default_git_depth: Option<u64>,
    /// Whether Auto DevOps are enabled or not.
    #[builder(default)]
    auto_devops_enabled: Option<bool>,
    /// The Auto Deploy strategy of the project.
    #[builder(default)]
    auto_devops_deploy_strategy: Option<AutoDevOpsDeployStrategy>,
    /// The storage shard on which to store the repository.
    #[builder(setter(into), default)]
    repository_storage: Option<Cow<'a, str>>,
    /// How many approvals are required before allowing merges.
    #[builder(default)]
    approvals_before_merge: Option<u64>,
    /// The classification label of the project.
    #[builder(setter(into), default)]
    external_authorization_classification_label: Option<Cow<'a, str>>,
    /// Whether to enable pull mirroring for the project or not.
    #[builder(default)]
    mirror: Option<bool>,
    /// User to attribute all mirror activity to.
    #[builder(default)]
    mirror_user_id: Option<u64>,
    /// Whether mirror updates trigger CI builds ir not.
    #[builder(default)]
    mirror_trigger_builds: Option<bool>,
    /// Whether to only mirror protected branches or not.
    #[builder(default)]
    only_mirror_protected_branches: Option<bool>,
    /// Whether the mirror overwrites diverged branches in this project or not.
    #[builder(default)]
    mirror_overwrites_diverged_branches: Option<bool>,
    /// Whether the package repository is enabled or not.
    #[builder(default)]
    packages_enabled: Option<bool>,
    /// Whether the service desk is enabled or not.
    #[builder(default)]
    service_desk_enabled: Option<bool>,

    /// Whether to enable issues or not.
    #[deprecated(note = "use `issues_access_level` instead")]
    #[builder(default)]
    issues_enabled: Option<bool>,
    /// Whether to enable merge requests or not.
    #[deprecated(note = "use `merge_requests_access_level` instead")]
    #[builder(default)]
    merge_requests_enabled: Option<bool>,
    /// Whether to enable CI pipelines or not.
    #[deprecated(note = "use `builds_access_level` instead")]
    #[builder(default)]
    jobs_enabled: Option<bool>,
    /// Whether to enable the wiki or not.
    #[deprecated(note = "use `wiki_access_level` instead")]
    #[builder(default)]
    wiki_enabled: Option<bool>,
    /// Whether to enable snippets or not.
    #[deprecated(note = "use `snippets_access_level` instead")]
    #[builder(default)]
    snippets_enabled: Option<bool>,
}

impl<'a> EditProject<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> EditProjectBuilder<'a> {
        EditProjectBuilder::default()
    }
}

impl<'a> EditProjectBuilder<'a> {
    /// Add a tag.
    pub fn tag<T>(&mut self, tag: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.tag_list
            .get_or_insert_with(HashSet::new)
            .insert(tag.into());
        self
    }

    /// Add multiple tags.
    pub fn tags<I, T>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = T>,
        T: Into<Cow<'a, str>>,
    {
        self.tag_list
            .get_or_insert_with(HashSet::new)
            .extend(iter.map(Into::into));
        self
    }
}

impl<'a> Endpoint for EditProject<'a> {
    fn method(&self) -> Method {
        Method::PUT
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        self.name
            .as_ref()
            .map(|value| pairs.append_pair("name", value));
        self.path
            .as_ref()
            .map(|value| pairs.append_pair("path", value));
        self.default_branch
            .as_ref()
            .map(|value| pairs.append_pair("default_branch", value));
        self.description
            .as_ref()
            .map(|value| pairs.append_pair("default_branch", value));

        self.issues_access_level
            .map(|value| pairs.append_pair("issues_access_level", value.as_str()));
        self.repository_access_level
            .map(|value| pairs.append_pair("repository_access_level", value.as_str()));
        self.merge_requests_access_level
            .map(|value| pairs.append_pair("merge_requests_access_level", value.as_str()));
        self.forking_access_level
            .map(|value| pairs.append_pair("forking_access_level", value.as_str()));
        self.builds_access_level
            .map(|value| pairs.append_pair("builds_access_level", value.as_str()));
        self.wiki_access_level
            .map(|value| pairs.append_pair("wiki_access_level", value.as_str()));
        self.snippets_access_level
            .map(|value| pairs.append_pair("snippets_access_level", value.as_str()));
        self.pages_access_level
            .map(|value| pairs.append_pair("pages_access_level", value.as_str()));

        self.emails_disabled
            .map(|value| pairs.append_pair("emails_disabled", common::bool_str(value)));
        self.resolve_outdated_diff_discussions.map(|value| {
            pairs.append_pair("resolve_outdated_diff_discussions", common::bool_str(value))
        });
        self.container_registry_enabled
            .map(|value| pairs.append_pair("container_registry_enabled", common::bool_str(value)));
        if let Some(policy) = self.container_expiration_policy_attributes.as_ref() {
            policy.add_query(&mut pairs);
        }
        self.shared_runners_enabled
            .map(|value| pairs.append_pair("shared_runners_enabled", common::bool_str(value)));
        self.visibility
            .map(|value| pairs.append_pair("visibility", value.as_str()));
        self.import_url
            .as_ref()
            .map(|value| pairs.append_pair("import_url", value));
        self.public_builds
            .map(|value| pairs.append_pair("public_builds", common::bool_str(value)));
        self.only_allow_merge_if_pipeline_succeeds.map(|value| {
            pairs.append_pair(
                "only_allow_merge_if_pipeline_succeeds",
                common::bool_str(value),
            )
        });
        self.only_allow_merge_if_all_discussions_are_resolved
            .map(|value| {
                pairs.append_pair(
                    "only_allow_merge_if_all_discussions_are_resolved",
                    common::bool_str(value),
                )
            });
        self.merge_method
            .map(|value| pairs.append_pair("merge_method", value.as_str()));
        self.autoclose_referenced_issues
            .map(|value| pairs.append_pair("autoclose_referenced_issues", common::bool_str(value)));
        self.suggestion_commit_message
            .as_ref()
            .map(|value| pairs.append_pair("suggestion_commit_message", value));
        self.remove_source_branch_after_merge.map(|value| {
            pairs.append_pair("remove_source_branch_after_merge", common::bool_str(value))
        });
        self.lfs_enabled
            .map(|value| pairs.append_pair("lfs_enabled", common::bool_str(value)));
        self.request_access_enabled
            .map(|value| pairs.append_pair("request_access_enabled", common::bool_str(value)));
        pairs.extend_pairs(self.tag_list.iter().map(|value| ("tag_list[]", value)));
        self.build_git_strategy
            .map(|value| pairs.append_pair("build_git_strategy", value.as_str()));
        self.build_timeout
            .map(|value| pairs.append_pair("build_timeout", &format!("{}", value)));
        self.auto_cancel_pending_pipelines
            .map(|value| pairs.append_pair("auto_cancel_pending_pipelines", value.as_str()));
        self.build_coverage_regex
            .as_ref()
            .map(|value| pairs.append_pair("build_coverage_regex", value));
        self.ci_config_path
            .as_ref()
            .map(|value| pairs.append_pair("ci_config_path", value));
        self.ci_default_git_depth
            .map(|value| pairs.append_pair("ci_default_git_depth", &format!("{}", value)));
        self.auto_devops_enabled
            .map(|value| pairs.append_pair("auto_devops_enabled", common::bool_str(value)));
        self.auto_devops_deploy_strategy
            .map(|value| pairs.append_pair("auto_devops_deploy_strategy", value.as_str()));
        self.repository_storage
            .as_ref()
            .map(|value| pairs.append_pair("repository_storage", value));
        self.approvals_before_merge
            .map(|value| pairs.append_pair("approvals_before_merge", &format!("{}", value)));
        self.external_authorization_classification_label
            .as_ref()
            .map(|value| pairs.append_pair("external_authorization_classification_label", value));
        self.mirror
            .map(|value| pairs.append_pair("mirror", common::bool_str(value)));
        self.mirror_user_id
            .map(|value| pairs.append_pair("mirror_user_id", &format!("{}", value)));
        self.mirror_trigger_builds
            .map(|value| pairs.append_pair("mirror_trigger_builds", common::bool_str(value)));
        self.only_mirror_protected_branches.map(|value| {
            pairs.append_pair("only_mirror_protected_branches", common::bool_str(value))
        });
        self.mirror_overwrites_diverged_branches.map(|value| {
            pairs.append_pair(
                "mirror_overwrites_diverged_branches",
                common::bool_str(value),
            )
        });
        self.packages_enabled
            .map(|value| pairs.append_pair("packages_enabled", common::bool_str(value)));
        self.service_desk_enabled
            .map(|value| pairs.append_pair("service_desk_enabled", common::bool_str(value)));

        #[allow(deprecated)]
        {
            self.issues_enabled
                .map(|value| pairs.append_pair("issues_enabled", common::bool_str(value)));
            self.merge_requests_enabled
                .map(|value| pairs.append_pair("merge_requests_enabled", common::bool_str(value)));
            self.jobs_enabled
                .map(|value| pairs.append_pair("jobs_enabled", common::bool_str(value)));
            self.wiki_enabled
                .map(|value| pairs.append_pair("wiki_enabled", common::bool_str(value)));
            self.snippets_enabled
                .map(|value| pairs.append_pair("snippets_enabled", common::bool_str(value)));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::EditProject;

    #[test]
    fn project_is_needed() {
        let err = EditProject::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_sufficient() {
        EditProject::builder().project("project").build().unwrap();
    }
}
