// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;
use std::collections::HashSet;

use derive_builder::Builder;

use crate::api::common::{self, EnableState, VisibilityLevel};
use crate::api::endpoint_prelude::*;

/// Access levels available for most features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureAccessLevel {
    /// The feature is not available at all.
    Disabled,
    /// The features is only available to project members.
    Private,
    /// The feature is available to everyone with access to the project.
    Enabled,
}

impl FeatureAccessLevel {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            FeatureAccessLevel::Disabled => "disabled",
            FeatureAccessLevel::Private => "private",
            FeatureAccessLevel::Enabled => "enabled",
        }
    }
}

/// Access levels available for features.
///
/// Note that only the `pages` feature currently uses this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureAccessLevelPublic {
    /// The feature is not available at all.
    Disabled,
    /// The features is only available to project members.
    Private,
    /// The feature is available to everyone with access to the project.
    Enabled,
    /// The feature is publicly available regardless of project access.
    Public,
}

impl FeatureAccessLevelPublic {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            FeatureAccessLevelPublic::Disabled => "disabled",
            FeatureAccessLevelPublic::Private => "private",
            FeatureAccessLevelPublic::Enabled => "enabled",
            FeatureAccessLevelPublic::Public => "public",
        }
    }
}

/// How often the container expiration policy is applied.
///
/// Note that GitLab only supports a few discrete values for this setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContainerExpirationCadence {
    /// Every day.
    OneDay,
    /// Every week.
    OneWeek,
    /// Every other week.
    TwoWeeks,
    /// Every month.
    OneMonth,
    /// Quaterly.
    ThreeMonths,
}

impl ContainerExpirationCadence {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContainerExpirationCadence::OneDay => "1d",
            ContainerExpirationCadence::OneWeek => "7d",
            ContainerExpirationCadence::TwoWeeks => "14d",
            ContainerExpirationCadence::OneMonth => "1month",
            ContainerExpirationCadence::ThreeMonths => "3month",
        }
    }
}

/// How many container instances to keep around.
///
/// Note that GitLab only supports a few discrete values for this setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContainerExpirationKeepN {
    /// Only one.
    One,
    /// Up to five.
    Five,
    /// Up to ten.
    Ten,
    /// Up to twenty-five.
    TwentyFive,
    /// Up to fifty.
    Fifty,
    /// Up to one hunder.
    OneHundred,
}

impl ContainerExpirationKeepN {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContainerExpirationKeepN::One => "1",
            ContainerExpirationKeepN::Five => "5",
            ContainerExpirationKeepN::Ten => "10",
            ContainerExpirationKeepN::TwentyFive => "25",
            ContainerExpirationKeepN::Fifty => "50",
            ContainerExpirationKeepN::OneHundred => "100",
        }
    }
}

/// How old containers need to be before they are candidates for expiration.
///
/// Note that GitLab only supports a few discrete values for this setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContainerExpirationOlderThan {
    /// One week old.
    OneWeek,
    /// Two weeks old.
    TwoWeeks,
    /// One month old.
    OneMonth,
    /// Three months old.
    ThreeMonths,
}

impl ContainerExpirationOlderThan {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContainerExpirationOlderThan::OneWeek => "7d",
            ContainerExpirationOlderThan::TwoWeeks => "14d",
            ContainerExpirationOlderThan::OneMonth => "30d",
            ContainerExpirationOlderThan::ThreeMonths => "90d",
        }
    }
}

/// The expiration policies for container images attached to the project.
#[derive(Debug, Clone, Builder)]
#[builder(setter(strip_option))]
pub struct ContainerExpirationPolicy<'a> {
    /// How often the policy should be applied.
    #[builder(default)]
    cadence: Option<ContainerExpirationCadence>,
    /// Whether the policy is enabled or not.
    #[builder(setter(into), default)]
    enabled: Option<bool>,
    /// How many container images to keep.
    #[builder(default)]
    keep_n: Option<ContainerExpirationKeepN>,
    /// Only consider containers older than this age.
    #[builder(default)]
    older_than: Option<ContainerExpirationOlderThan>,
    /// Only apply to images with names maching a regular expression.
    ///
    /// See the [Ruby documentation](https://ruby-doc.org/core-2.7.1/Regexp.html) for supported
    /// syntax.
    #[builder(setter(into), default)]
    name_regex: Option<Cow<'a, str>>,
}

impl<'a> ContainerExpirationPolicy<'a> {
    /// Create a builder for the container expiration policy.
    pub fn builder() -> ContainerExpirationPolicyBuilder<'a> {
        ContainerExpirationPolicyBuilder::default()
    }

    pub(crate) fn add_query(&self, pairs: &mut Pairs) {
        self.cadence.map(|value| {
            pairs.append_pair(
                "container_expiration_policy_attributes[cadence]",
                value.as_str(),
            )
        });
        self.enabled.map(|value| {
            pairs.append_pair(
                "container_expiration_policy_attributes[enabled]",
                common::bool_str(value),
            )
        });
        self.keep_n.map(|value| {
            pairs.append_pair(
                "container_expiration_policy_attributes[keep_n]",
                value.as_str(),
            )
        });
        self.older_than.map(|value| {
            pairs.append_pair(
                "container_expiration_policy_attributes[older_than]",
                value.as_str(),
            )
        });
        self.name_regex.as_ref().map(|value| {
            pairs.append_pair("container_expiration_policy_attributes[name_regex]", value)
        });
    }
}

/// The deploy strategy used when Auto DevOps is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoDevOpsDeployStrategy {
    /// Continuous deployment.
    Continuous,
    /// Manual deployment.
    Manual,
    /// Interval deployments.
    TimedIncremental,
}

impl AutoDevOpsDeployStrategy {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            AutoDevOpsDeployStrategy::Continuous => "continuous",
            AutoDevOpsDeployStrategy::Manual => "manual",
            AutoDevOpsDeployStrategy::TimedIncremental => "timed_incremental",
        }
    }
}

/// How merge requests should be merged when using the "Merge" button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMethod {
    /// Always create a merge commit.
    Merge,
    /// Always create a merge commit, but require that the branch be fast-forward capable.
    RebaseMerge,
    /// Only fast-forward merges are allowed.
    FastForward,
}

impl MergeMethod {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            MergeMethod::Merge => "merge",
            MergeMethod::RebaseMerge => "rebase_merge",
            MergeMethod::FastForward => "ff",
        }
    }
}

/// The default Git strategy for CI jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildGitStrategy {
    /// Clone the reopsitory every time.
    Clone,
    /// Fetch into an existing checkout (will clone if not available).
    Fetch,
    /// Do not update the repository at all.
    None,
}

impl Default for BuildGitStrategy {
    fn default() -> Self {
        BuildGitStrategy::Fetch
    }
}

impl BuildGitStrategy {
    /// The variable type query parameter.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            BuildGitStrategy::Clone => "clone",
            BuildGitStrategy::Fetch => "fetch",
            BuildGitStrategy::None => "none",
        }
    }
}

/// A structure to handle the fact that at least one of the name and path is required.
#[derive(Debug, Clone)]
enum ProjectName<'a> {
    /// The name of the new project.
    ///
    /// The `path` is based on the name.
    Name { name: Cow<'a, str> },
    /// The path of the new project.
    ///
    /// The `name` is the path.
    Path { path: Cow<'a, str> },
    /// Provide both the name and path manually.
    NameAndPath {
        name: Cow<'a, str>,
        path: Cow<'a, str>,
    },
}

impl<'a> ProjectName<'a> {
    fn with_name(self, name: Cow<'a, str>) -> Self {
        match self {
            ProjectName::Name {
                ..
            } => {
                ProjectName::Name {
                    name,
                }
            },
            ProjectName::NameAndPath {
                path, ..
            }
            | ProjectName::Path {
                path,
            } => {
                ProjectName::NameAndPath {
                    name,
                    path,
                }
            },
        }
    }

    fn with_path(self, path: Cow<'a, str>) -> Self {
        match self {
            ProjectName::Path {
                ..
            } => {
                ProjectName::Path {
                    path,
                }
            },
            ProjectName::NameAndPath {
                name, ..
            }
            | ProjectName::Name {
                name,
            } => {
                ProjectName::NameAndPath {
                    name,
                    path,
                }
            },
        }
    }
}

/// Create a new project on an instance.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateProject<'a> {
    /// The name and/or path of the project.
    #[builder(private)]
    name_and_path: ProjectName<'a>,
    /// The namespace of the new project.
    ///
    /// By default, the project is created in the API caller's namespace.
    #[builder(default)]
    namespace_id: Option<u64>,
    /// The default branch of the new project.
    ///
    /// Defaults to `master`.
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
    /// Whether to show a link to create or view a merge request when pushing a branch from the
    /// command line or not.
    #[builder(default)]
    printing_merge_request_link_enabled: Option<bool>,
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
    /// Whether mirror updates trigger CI builds ir not.
    #[builder(default)]
    mirror_trigger_builds: Option<bool>,
    /// Initialize the project with a readme.
    #[builder(default)]
    initialize_with_readme: Option<bool>,
    /// The name of a template project to use.
    #[builder(setter(into), default)]
    template_name: Option<Cow<'a, str>>,
    /// The ID of the template project to use.
    #[builder(default)]
    template_project_id: Option<u64>,
    /// Whether to use a custom instance or group template.
    #[builder(default)]
    use_custom_template: Option<bool>,
    /// Whether the template project should come from the group or the instance.
    #[builder(setter(name = "_group_with_project_templates_id"), default, private)]
    group_with_project_templates_id: Option<u64>,
    /// Whether the package repository is enabled or not.
    #[builder(default)]
    packages_enabled: Option<bool>,

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

impl<'a> CreateProject<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateProjectBuilder<'a> {
        CreateProjectBuilder::default()
    }
}

impl<'a> CreateProjectBuilder<'a> {
    /// Set the name of the project.
    ///
    /// If not set, it will default to the value of `path`.
    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: Into<Cow<'a, str>>,
    {
        let name = name.into();
        self.name_and_path = Some(if let Some(name_and_path) = self.name_and_path.take() {
            name_and_path.with_name(name)
        } else {
            ProjectName::Name {
                name,
            }
        });
        self
    }

    /// Set the path of the project.
    ///
    /// If not set, it will default to the value of `name` after processing to make it a valid
    /// path.
    pub fn path<P>(&mut self, path: P) -> &mut Self
    where
        P: Into<Cow<'a, str>>,
    {
        let path = path.into();
        self.name_and_path = Some(if let Some(name_and_path) = self.name_and_path.take() {
            name_and_path.with_path(path)
        } else {
            ProjectName::Path {
                path,
            }
        });
        self
    }

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

    /// Whether the template project should come from the group or the instance.
    ///
    /// Note that setting this also sets `use_custom_template` to `true` automatically.
    pub fn group_with_project_templates_id(&mut self, id: u64) -> &mut Self {
        self.group_with_project_templates_id = Some(Some(id));
        self.use_custom_template(true);
        self
    }
}

impl<'a> Endpoint for CreateProject<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        "projects".into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        match &self.name_and_path {
            ProjectName::Name {
                name,
            } => {
                pairs.append_pair("name", &name);
            },
            ProjectName::Path {
                path,
            } => {
                pairs.append_pair("path", &path);
            },
            ProjectName::NameAndPath {
                name,
                path,
            } => {
                pairs.append_pair("name", &name);
                pairs.append_pair("path", &path);
            },
        }

        self.namespace_id
            .map(|value| pairs.append_pair("namespace_id", &format!("{}", value)));
        self.default_branch
            .as_ref()
            .map(|value| pairs.append_pair("default_branch", value));
        self.description
            .as_ref()
            .map(|value| pairs.append_pair("description", value));

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
        self.remove_source_branch_after_merge.map(|value| {
            pairs.append_pair("remove_source_branch_after_merge", common::bool_str(value))
        });
        self.lfs_enabled
            .map(|value| pairs.append_pair("lfs_enabled", common::bool_str(value)));
        self.request_access_enabled
            .map(|value| pairs.append_pair("request_access_enabled", common::bool_str(value)));
        pairs.extend_pairs(self.tag_list.iter().map(|value| ("tag_list[]", value)));
        self.printing_merge_request_link_enabled.map(|value| {
            pairs.append_pair(
                "printing_merge_request_link_enabled",
                common::bool_str(value),
            )
        });
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
        self.mirror_trigger_builds
            .map(|value| pairs.append_pair("mirror_trigger_builds", common::bool_str(value)));
        self.initialize_with_readme
            .map(|value| pairs.append_pair("initialize_with_readme", common::bool_str(value)));
        self.template_name
            .as_ref()
            .map(|value| pairs.append_pair("template_name", value));
        self.template_project_id
            .map(|value| pairs.append_pair("template_project_id", &format!("{}", value)));
        self.use_custom_template
            .map(|value| pairs.append_pair("use_custom_template", common::bool_str(value)));
        self.group_with_project_templates_id.map(|value| {
            pairs.append_pair("group_with_project_templates_id", &format!("{}", value))
        });
        self.packages_enabled
            .map(|value| pairs.append_pair("packages_enabled", common::bool_str(value)));

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
    use crate::api::projects::{
        AutoDevOpsDeployStrategy, BuildGitStrategy, ContainerExpirationCadence,
        ContainerExpirationKeepN, ContainerExpirationOlderThan, ContainerExpirationPolicy,
        CreateProject, FeatureAccessLevel, FeatureAccessLevelPublic, MergeMethod,
    };

    #[test]
    fn feature_access_level_as_str() {
        let items = &[
            (FeatureAccessLevel::Disabled, "disabled"),
            (FeatureAccessLevel::Private, "private"),
            (FeatureAccessLevel::Enabled, "enabled"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn feature_access_level_public_as_str() {
        let items = &[
            (FeatureAccessLevelPublic::Disabled, "disabled"),
            (FeatureAccessLevelPublic::Private, "private"),
            (FeatureAccessLevelPublic::Enabled, "enabled"),
            (FeatureAccessLevelPublic::Public, "public"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn container_expiration_cadence_as_str() {
        let items = &[
            (ContainerExpirationCadence::OneDay, "1d"),
            (ContainerExpirationCadence::OneWeek, "7d"),
            (ContainerExpirationCadence::TwoWeeks, "14d"),
            (ContainerExpirationCadence::OneMonth, "1month"),
            (ContainerExpirationCadence::ThreeMonths, "3month"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn container_expiration_keep_n_ordering() {
        let items = &[
            ContainerExpirationKeepN::One,
            ContainerExpirationKeepN::Five,
            ContainerExpirationKeepN::Ten,
            ContainerExpirationKeepN::TwentyFive,
            ContainerExpirationKeepN::Fifty,
            ContainerExpirationKeepN::OneHundred,
        ];

        let mut last = None;
        for item in items {
            if let Some(prev) = last {
                assert!(prev < item);
            }
            last = Some(item);
        }
    }

    #[test]
    fn container_expiration_keep_n_as_str() {
        let items = &[
            (ContainerExpirationKeepN::One, "1"),
            (ContainerExpirationKeepN::Five, "5"),
            (ContainerExpirationKeepN::Ten, "10"),
            (ContainerExpirationKeepN::TwentyFive, "25"),
            (ContainerExpirationKeepN::Fifty, "50"),
            (ContainerExpirationKeepN::OneHundred, "100"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn container_expiration_older_than_ordering() {
        let items = &[
            ContainerExpirationOlderThan::OneWeek,
            ContainerExpirationOlderThan::TwoWeeks,
            ContainerExpirationOlderThan::OneMonth,
            ContainerExpirationOlderThan::ThreeMonths,
        ];

        let mut last = None;
        for item in items {
            if let Some(prev) = last {
                assert!(prev < item);
            }
            last = Some(item);
        }
    }

    #[test]
    fn container_expiration_older_than_as_str() {
        let items = &[
            (ContainerExpirationOlderThan::OneWeek, "7d"),
            (ContainerExpirationOlderThan::TwoWeeks, "14d"),
            (ContainerExpirationOlderThan::OneMonth, "30d"),
            (ContainerExpirationOlderThan::ThreeMonths, "90d"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn container_expiration_policy_default() {
        ContainerExpirationPolicy::builder().build().unwrap();
    }

    #[test]
    fn auto_dev_ops_deploy_strategy_as_str() {
        let items = &[
            (AutoDevOpsDeployStrategy::Continuous, "continuous"),
            (AutoDevOpsDeployStrategy::Manual, "manual"),
            (
                AutoDevOpsDeployStrategy::TimedIncremental,
                "timed_incremental",
            ),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn merge_method_as_str() {
        let items = &[
            (MergeMethod::Merge, "merge"),
            (MergeMethod::RebaseMerge, "rebase_merge"),
            (MergeMethod::FastForward, "ff"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn build_git_strategy_default() {
        assert_eq!(BuildGitStrategy::default(), BuildGitStrategy::Fetch);
    }

    #[test]
    fn build_git_strategy_as_str() {
        let items = &[
            (BuildGitStrategy::Clone, "clone"),
            (BuildGitStrategy::Fetch, "fetch"),
            (BuildGitStrategy::None, "none"),
        ];

        for (i, s) in items {
            assert_eq!(i.as_str(), *s);
        }
    }

    #[test]
    fn name_and_path_is_needed() {
        let err = CreateProject::builder().build().unwrap_err();
        assert_eq!(err, "`name_and_path` must be initialized");
    }

    #[test]
    fn name_is_sufficient() {
        CreateProject::builder().name("name").build().unwrap();
    }

    #[test]
    fn path_is_sufficient() {
        CreateProject::builder().path("path").build().unwrap();
    }
}
