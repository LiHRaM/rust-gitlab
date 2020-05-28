// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeSet;

use derive_builder::Builder;

use crate::api::common::{EnableState, NameOrId, VisibilityLevel};
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
    #[builder(setter(into), default)]
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
    tag_list: BTreeSet<Cow<'a, str>>,
    // TODO: Figure out how to actually use this.
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
    #[builder(setter(into), default)]
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
            .get_or_insert_with(BTreeSet::new)
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
            .get_or_insert_with(BTreeSet::new)
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

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push_opt("name", self.name.as_ref())
            .push_opt("path", self.path.as_ref())
            .push_opt("default_branch", self.default_branch.as_ref())
            .push_opt("description", self.description.as_ref())
            .push_opt("issues_access_level", self.issues_access_level)
            .push_opt("repository_access_level", self.repository_access_level)
            .push_opt(
                "merge_requests_access_level",
                self.merge_requests_access_level,
            )
            .push_opt("forking_access_level", self.forking_access_level)
            .push_opt("builds_access_level", self.builds_access_level)
            .push_opt("wiki_access_level", self.wiki_access_level)
            .push_opt("snippets_access_level", self.snippets_access_level)
            .push_opt("pages_access_level", self.pages_access_level)
            .push_opt("emails_disabled", self.emails_disabled)
            .push_opt(
                "resolve_outdated_diff_discussions",
                self.resolve_outdated_diff_discussions,
            )
            .push_opt(
                "container_registry_enabled",
                self.container_registry_enabled,
            )
            .push_opt("shared_runners_enabled", self.shared_runners_enabled)
            .push_opt("visibility", self.visibility)
            .push_opt("import_url", self.import_url.as_ref())
            .push_opt("public_builds", self.public_builds)
            .push_opt(
                "only_allow_merge_if_pipeline_succeeds",
                self.only_allow_merge_if_pipeline_succeeds,
            )
            .push_opt(
                "only_allow_merge_if_all_discussions_are_resolved",
                self.only_allow_merge_if_all_discussions_are_resolved,
            )
            .push_opt("merge_method", self.merge_method)
            .push_opt(
                "autoclose_referenced_issues",
                self.autoclose_referenced_issues,
            )
            .push_opt(
                "suggestion_commit_message",
                self.suggestion_commit_message.as_ref(),
            )
            .push_opt(
                "remove_source_branch_after_merge",
                self.remove_source_branch_after_merge,
            )
            .push_opt("lfs_enabled", self.lfs_enabled)
            .push_opt("request_access_enabled", self.request_access_enabled)
            .extend(self.tag_list.iter().map(|value| ("tag_list[]", value)))
            .push_opt("build_git_strategy", self.build_git_strategy)
            .push_opt("build_timeout", self.build_timeout)
            .push_opt(
                "auto_cancel_pending_pipelines",
                self.auto_cancel_pending_pipelines,
            )
            .push_opt("build_coverage_regex", self.build_coverage_regex.as_ref())
            .push_opt("ci_config_path", self.ci_config_path.as_ref())
            .push_opt("ci_default_git_depth", self.ci_default_git_depth)
            .push_opt("auto_devops_enabled", self.auto_devops_enabled)
            .push_opt(
                "auto_devops_deploy_strategy",
                self.auto_devops_deploy_strategy,
            )
            .push_opt("repository_storage", self.repository_storage.as_ref())
            .push_opt("approvals_before_merge", self.approvals_before_merge)
            .push_opt(
                "external_authorization_classification_label",
                self.external_authorization_classification_label.as_ref(),
            )
            .push_opt("mirror", self.mirror)
            .push_opt("mirror_user_id", self.mirror_user_id)
            .push_opt("mirror_trigger_builds", self.mirror_trigger_builds)
            .push_opt(
                "only_mirror_protected_branches",
                self.only_mirror_protected_branches,
            )
            .push_opt(
                "mirror_overwrites_diverged_branches",
                self.mirror_overwrites_diverged_branches,
            )
            .push_opt("packages_enabled", self.packages_enabled)
            .push_opt("service_desk_enabled", self.service_desk_enabled);

        if let Some(policy) = self.container_expiration_policy_attributes.as_ref() {
            policy.add_query(&mut params);
        }

        #[allow(deprecated)]
        {
            params
                .push_opt("issues_enabled", self.issues_enabled)
                .push_opt("merge_requests_enabled", self.merge_requests_enabled)
                .push_opt("jobs_enabled", self.jobs_enabled)
                .push_opt("wiki_enabled", self.wiki_enabled)
                .push_opt("snippets_enabled", self.snippets_enabled);
        }

        params.into_body()
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
