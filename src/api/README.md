# Implemented

These API endpoints have been implemented.

  * `GET    /projects` `projects/projects.rs`
  * `POST   /projects` `projects/projects/create.rs`
  * `PUT    /projects/:project` `projects/projects/edit.rs`
  * `GET    /projects/:project/jobs` `projects/jobs.rs`
  * `GET    /projects/:project/job/:id` `projects/job.rs`
  * `GET    /projects/:project/pipeline` `projects/pipelines/create.rs`
  * `GET    /projects/:project/pipelines` `projects/pipelines/pipelines.rs`
  * `GET    /projects/:project/pipelines/:pipeline` `projects/pipelines/pipeline.rs`
  * `DELETE /projects/:project/pipelines/:pipeline` `projects/pipelines/delete.rs`
  * `POST   /projects/:project/pipelines/:pipeline/cancel` `projects/pipelines/cancel.rs`
  * `GET    /projects/:project/pipelines/:pipeline/jobs` `projects/pipelines/jobs/jobs.rs`
  * `POST   /projects/:project/pipelines/:pipeline/retry` `projects/pipelines/retry.rs`
  * `GET    /projects/:project/pipelines/:pipeline/variables` `projects/pipelines/variables.rs`
  * `GET    /user` `users/current_user.rs`
  * `GET    /users` `users/users.rs`
  * `GET    /users/:user` `users/user.rs`

# Todo

This section contains the list of API endpoints which are not yet implemented
in this crate. Contributions welcome!

## Specific endpoints

These endpoints are documented on a page that have other endpoints already
implemented above. This is split out into a separate list for convenience
instead of having to search the page for missing endpoints.

  * `GET    /projects/:project` https://gitlab.kitware.com/help/api/projects.md#get-single-project
  * `DELETE /projects/:project` https://gitlab.kitware.com/help/api/projects.md#remove-project
  * `POST   /projects/:project/archive` https://gitlab.kitware.com/help/api/projects.md#archive-a-project
  * `POST   /projects/:project/unarchive` https://gitlab.kitware.com/help/api/projects.md#unarchive-a-project
  * `POST   /projects/:project/fork` https://gitlab.kitware.com/help/api/projects.md#fork-project
  * `DELETE /projects/:project/fork` https://gitlab.kitware.com/help/api/projects.md#delete-an-existing-forked-from-relationship
  * `POST   /projects/:project/fork/:from` https://gitlab.kitware.com/help/api/projects.md#create-a-forked-fromto-relation-between-existing-projects
  * `GET    /projects/:project/forks` https://gitlab.kitware.com/help/api/projects.md#list-forks-of-a-project
  * `GET    /projects/:project/hooks` https://gitlab.kitware.com/help/api/projects.md#list-project-hooks
  * `POST   /projects/:project/hooks` https://gitlab.kitware.com/help/api/projects.md#add-project-hook
  * `GET    /projects/:project/hooks/:id` https://gitlab.kitware.com/help/api/projects.md#get-project-hook
  * `PUT    /projects/:project/hooks/:id` https://gitlab.kitware.com/help/api/projects.md#edit-project-hook
  * `DELETE /projects/:project/hooks/:id` https://gitlab.kitware.com/help/api/projects.md#delete-project-hook
  * `POST   /projects/:project/housekeeping` https://gitlab.kitware.com/help/api/projects.md#start-the-housekeeping-task-for-a-project
  * `GET    /projects/:project/jobs/:id/artifacts/*artifact_path` https://gitlab.kitware.com/help/api/jobs.md#download-a-single-artifact-file-by-job-id
  * `GET    /projects/:project/jobs/:id/artifacts` https://gitlab.kitware.com/help/api/jobs.md#get-job-artifacts
  * `DELETE /projects/:project/jobs/:id/artifacts` https://gitlab.kitware.com/help/api/jobs.md#delete-artifacts
  * `POST   /projects/:project/jobs/:id/artifacts/keep` https://gitlab.kitware.com/help/api/jobs.md#keep-artifacts
  * `POST   /projects/:project/jobs/:id/cancel` https://gitlab.kitware.com/help/api/jobs.md#cancel-a-job
  * `POST   /projects/:project/jobs/:id/erase` https://gitlab.kitware.com/help/api/jobs.md#erase-a-job
  * `POST   /projects/:project/jobs/:id/retry` https://gitlab.kitware.com/help/api/jobs.md#retry-a-job
  * `POST   /projects/:project/jobs/:id/play` https://gitlab.kitware.com/help/api/jobs.md#play-a-job
  * `GET    /projects/:project/jobs/:id/trace` https://gitlab.kitware.com/help/api/jobs.md#get-a-log-file
  * `GET    /projects/:project/jobs/artifacts/:ref/download` https://gitlab.kitware.com/help/api/jobs.md#download-the-artifacts-archive
  * `GET    /projects/:project/jobs/artifacts/:ref/raw/*artifact_path` https://gitlab.kitware.com/help/api/jobs.md#download-a-single-artifact-file-from-specific-tag-or-branch
  * `GET    /projects/:project/languages` https://gitlab.kitware.com/help/api/projects.md#languages
  * `POST   /projects/:project/mirror/pull` https://gitlab.kitware.com/help/api/projects.md#start-the-pull-mirroring-process-for-a-project-starter
  * `POST   /projects/:project/pipeline` https://gitlab.kitware.com/help/api/pipelines.md#create-a-new-pipeline
  * `GET    /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#get-project-push-rules
  * `POST   /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#add-project-push-rule
  * `PUT    /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#edit-project-push-rule
  * `DELETE /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#delete-project-push-rule
  * `POST   /projects/:project/restore` https://gitlab.kitware.com/help/api/projects.md#restore-project-marked-for-deletion-premium
  * `POST   /projects/:project/share` https://gitlab.kitware.com/help/api/projects.md#share-project-with-group
  * `DELETE /projects/:project/share/:group` https://gitlab.kitware.com/help/api/projects.md#delete-a-shared-project-link-within-a-group
  * `GET    /projects/:project/snapshot` https://gitlab.kitware.com/help/api/projects.md#download-snapshot-of-a-git-repository
  * `POST   /projects/:project/star` https://gitlab.kitware.com/help/api/projects.md#star-a-project
  * `GET    /projects/:project/starrers` https://gitlab.kitware.com/help/api/projects.md#list-starrers-of-a-project
  * `PUT    /projects/:project/transfer` https://gitlab.kitware.com/help/api/projects.md#transfer-a-project-to-a-new-namespace
  * `POST   /projects/:project/unstar` https://gitlab.kitware.com/help/api/projects.md#unstar-a-project
  * `POST   /projects/:project/upload` https://gitlab.kitware.com/help/api/projects.md#upload-a-file
  * `GET    /projects/:project/users` https://gitlab.kitware.com/help/api/projects.md#get-project-users
  * `POST   /projects/user/:user` https://gitlab.kitware.com/help/api/projects.md#create-project-for-user
  * `GET    /user/activities` https://gitlab.kitware.com/help/api/users.md#get-user-activities-admin-only
  * `GET    /user/emails` https://gitlab.kitware.com/help/api/users.md#list-emails
  * `POST   /user/emails` https://gitlab.kitware.com/help/api/users.md#add-email
  * `GET    /user/emails/:id` https://gitlab.kitware.com/help/api/users.md#single-email
  * `DELETE /user/emails/:id` https://gitlab.kitware.com/help/api/users.md#delete-email-for-current-user
  * `GET    /user/gpg_keys` https://gitlab.kitware.com/help/api/users.md#list-all-gpg-keys
  * `POST   /user/gpg_keys` https://gitlab.kitware.com/help/api/users.md#add-a-gpg-key
  * `GET    /user/gpg_keys/:id` https://gitlab.kitware.com/help/api/users.md#get-a-specific-gpg-key
  * `DELETE /user/gpg_keys/:id` https://gitlab.kitware.com/help/api/users.md#delete-a-gpg-key
  * `GET    /user/keys` https://gitlab.kitware.com/help/api/users.md#list-user-projects
  * `POST   /user/keys` https://gitlab.kitware.com/help/api/users.md#add-ssh-key
  * `GET    /user/keys/:id` https://gitlab.kitware.com/help/api/users.md#single-ssh-key
  * `DELETE /user/keys/:id` https://gitlab.kitware.com/help/api/users.md#delete-ssh-key-for-current-user
  * `GET    /user/status` https://gitlab.kitware.com/help/api/users.md#user-status
  * `PUT    /user/status` https://gitlab.kitware.com/help/api/users.md#set-user-status
  * `GET    /user_counts` https://gitlab.kitware.com/help/api/users.md#user-counts
  * `POST   /users` https://gitlab.kitware.com/help/api/users.md#user-creation
  * `DELETE /users/:user` https://gitlab.kitware.com/help/api/users.md#user-deletion
  * `PUT    /users/:user` https://gitlab.kitware.com/help/api/users.md#user-modification
  * `POST   /users/:user/activate` https://gitlab.kitware.com/help/api/users.md#activate-user
  * `POST   /users/:user/block` https://gitlab.kitware.com/help/api/users.md#block-user
  * `POST   /users/:user/deactivate` https://gitlab.kitware.com/help/api/users.md#deactivate-user
  * `GET    /users/:user/emails` https://gitlab.kitware.com/help/api/users.md#list-emails-for-user
  * `POST   /users/:user/emails` https://gitlab.kitware.com/help/api/users.md#add-email-for-user
  * `DELETE /users/:user/emails/:id` https://gitlab.kitware.com/help/api/users.md#delete-email-for-given-user
  * `GET    /users/:user/gpg_keys` https://gitlab.kitware.com/help/api/users.md#list-all-gpg-keys-for-given-user
  * `POST   /users/:user/gpg_keys` https://gitlab.kitware.com/help/api/users.md#add-a-gpg-key-for-a-given-user
  * `GET    /users/:user/gpg_keys/:id` https://gitlab.kitware.com/help/api/users.md#get-a-specific-gpg-key-for-a-given-user
  * `DELETE /users/:user/gpg_keys/:id` https://gitlab.kitware.com/help/api/users.md#add-a-gpg-key-for-a-given-user
  * `DELETE /users/:user/identities/:provider` https://gitlab.kitware.com/help/api/users.md#delete-authentication-identity-from-user
  * `GET    /users/:user/impersonation_tokens` https://gitlab.kitware.com/help/api/users.md#get-all-impersonation-tokens-of-a-user
  * `POST   /users/:user/impersonation_tokens` https://gitlab.kitware.com/help/api/users.md#create-an-impersonation-token
  * `GET    /users/:user/impersonation_tokens/:id` https://gitlab.kitware.com/help/api/users.md#get-an-impersonation-token-of-a-user
  * `DELETE /users/:user/impersonation_tokens/:id` https://gitlab.kitware.com/help/api/users.md#revoke-an-impersonation-token
  * `GET    /users/:user/keys` https://gitlab.kitware.com/help/api/users.md#list-ssh-keys-for-user
  * `POST   /users/:user/keys` https://gitlab.kitware.com/help/api/users.md#add-ssh-key-for-user
  * `DELETE /users/:user/keys/:id` https://gitlab.kitware.com/help/api/users.md#delete-ssh-key-for-given-user
  * `GET    /users/:user/memberships` https://gitlab.kitware.com/help/api/users.md#user-memberships-admin-only
  * `GET    /users/:user/projects` https://gitlab.kitware.com/help/api/projects.md#list-user-projects
  * `GET    /users/:user/starred_projects` https://gitlab.kitware.com/help/api/projects.md#list-projects-starred-by-a-user
  * `GET    /users/:user/status` https://gitlab.kitware.com/help/api/users.md#get-the-status-of-a-user
  * `POST   /users/:user/unblock` https://gitlab.kitware.com/help/api/users.md#unblock-user

## Endpoint groups

These pages document other endpoints not mentioned above:

  * https://gitlab.kitware.com/help/api/access_requests.md
  * https://gitlab.kitware.com/help/api/award_emoji.md
  * https://gitlab.kitware.com/help/api/branches.md
  * https://gitlab.kitware.com/help/api/commits.md
  * https://gitlab.kitware.com/help/api/container_registry.md
  * https://gitlab.kitware.com/help/api/custom_attributes.md
  * https://gitlab.kitware.com/help/api/dependencies.md
  * https://gitlab.kitware.com/help/api/deploy_keys.md
  * https://gitlab.kitware.com/help/api/deployments.md
  * https://gitlab.kitware.com/help/api/discussions.md
  * https://gitlab.kitware.com/help/api/environments.md
  * https://gitlab.kitware.com/help/api/error_tracking.md
  * https://gitlab.kitware.com/help/api/events.md
  * https://gitlab.kitware.com/help/api/issues.md
  * https://gitlab.kitware.com/help/api/issues_statistics.md
  * https://gitlab.kitware.com/help/api/boards.md
  * https://gitlab.kitware.com/help/api/issue_links.md
  * https://gitlab.kitware.com/help/api/labels.md
  * https://gitlab.kitware.com/help/api/managed_licenses.md
  * https://gitlab.kitware.com/help/api/members.md
  * https://gitlab.kitware.com/help/api/merge_request_approvals.md
  * https://gitlab.kitware.com/help/api/merge_requests.md
  * https://gitlab.kitware.com/help/api/notes.md
  * https://gitlab.kitware.com/help/api/notification_settings.md
  * https://gitlab.kitware.com/help/api/packages.md
  * https://gitlab.kitware.com/help/api/pages_domains.md
  * https://gitlab.kitware.com/help/api/pipeline_schedules.md
  * https://gitlab.kitware.com/help/api/pipeline_triggers.md
  * https://gitlab.kitware.com/help/api/project_badges.md
  * https://gitlab.kitware.com/help/api/project_clusters.md
  * https://gitlab.kitware.com/help/api/project_level_variables.md
  * https://gitlab.kitware.com/help/api/project_import_export.md
  * https://gitlab.kitware.com/help/api/milestones.md
  * https://gitlab.kitware.com/help/api/project_snippets.md
  * https://gitlab.kitware.com/help/api/project_templates.md
  * https://gitlab.kitware.com/help/api/protected_environments.md
  * https://gitlab.kitware.com/help/api/protected_branches.md
  * https://gitlab.kitware.com/help/api/protected_tags.md
  * https://gitlab.kitware.com/help/api/releases/index.md
  * https://gitlab.kitware.com/help/api/releases/links.md
  * https://gitlab.kitware.com/help/api/remote_mirrors.md
  * https://gitlab.kitware.com/help/api/repositories.md
  * https://gitlab.kitware.com/help/api/repository_files.md
  * https://gitlab.kitware.com/help/api/repository_submodules.md
  * https://gitlab.kitware.com/help/api/resource_label_events.md
  * https://gitlab.kitware.com/help/api/runners.md
  * https://gitlab.kitware.com/help/api/search.md
  * https://gitlab.kitware.com/help/api/services.md
  * https://gitlab.kitware.com/help/api/tags.md
  * https://gitlab.kitware.com/help/api/visual_review_discussions.md
  * https://gitlab.kitware.com/help/api/vulnerabilities.md
  * https://gitlab.kitware.com/help/api/vulnerability_exports.md
  * https://gitlab.kitware.com/help/api/project_vulnerabilities.md
  * https://gitlab.kitware.com/help/api/vulnerability_findings.md
  * https://gitlab.kitware.com/help/api/wikis.md
  * https://gitlab.kitware.com/help/api/access_requests.md
  * https://gitlab.kitware.com/help/api/custom_attributes.md
  * https://gitlab.kitware.com/help/api/discussions.md
  * https://gitlab.kitware.com/help/api/epic_issues.md
  * https://gitlab.kitware.com/help/api/epic_links.md
  * https://gitlab.kitware.com/help/api/epics.md
  * https://gitlab.kitware.com/help/api/groups.md
  * https://gitlab.kitware.com/help/api/group_badges.md
  * https://gitlab.kitware.com/help/api/group_boards.md
  * https://gitlab.kitware.com/help/api/group_labels.md
  * https://gitlab.kitware.com/help/api/group_level_variables.md
  * https://gitlab.kitware.com/help/api/group_milestones.md
  * https://gitlab.kitware.com/help/api/issues.md
  * https://gitlab.kitware.com/help/api/issues_statistics.md
  * https://gitlab.kitware.com/help/api/members.md
  * https://gitlab.kitware.com/help/api/merge_requests.md
  * https://gitlab.kitware.com/help/api/notes.md
  * https://gitlab.kitware.com/help/api/notification_settings.md
  * https://gitlab.kitware.com/help/api/resource_label_events.md
  * https://gitlab.kitware.com/help/api/search.md
  * https://gitlab.kitware.com/help/api/admin_sidekiq_queues.md
  * https://gitlab.kitware.com/help/api/appearance.md
  * https://gitlab.kitware.com/help/api/applications.md
  * https://gitlab.kitware.com/help/api/audit_events.md
  * https://gitlab.kitware.com/help/api/avatar.md
  * https://gitlab.kitware.com/help/api/broadcast_messages.md
  * https://gitlab.kitware.com/help/api/snippets.md
  * https://gitlab.kitware.com/help/api/custom_attributes.md
  * https://gitlab.kitware.com/help/api/deploy_keys.md
  * https://gitlab.kitware.com/help/api/events.md
  * https://gitlab.kitware.com/help/api/features.md
  * https://gitlab.kitware.com/help/api/geo_nodes.md
  * https://gitlab.kitware.com/help/api/group_activity_analytics.md
  * https://gitlab.kitware.com/help/api/import.md
  * https://gitlab.kitware.com/help/api/issues.md
  * https://gitlab.kitware.com/help/api/issues_statistics.md
  * https://gitlab.kitware.com/help/api/keys.md
  * https://gitlab.kitware.com/help/api/license.md
  * https://gitlab.kitware.com/help/api/markdown.md
  * https://gitlab.kitware.com/help/api/merge_requests.md
  * https://gitlab.kitware.com/help/api/namespaces.md
  * https://gitlab.kitware.com/help/api/notification_settings.md
  * https://gitlab.kitware.com/help/api/pages_domains.md
  * https://gitlab.kitware.com/help/api/projects.md
  * https://gitlab.kitware.com/help/api/runners.md
  * https://gitlab.kitware.com/help/api/search.md
  * https://gitlab.kitware.com/help/api/settings.md
  * https://gitlab.kitware.com/help/api/statistics.md
  * https://gitlab.kitware.com/help/api/sidekiq_metrics.md
  * https://gitlab.kitware.com/help/api/suggestions.md
  * https://gitlab.kitware.com/help/api/system_hooks.md
  * https://gitlab.kitware.com/help/api/todos.md
  * https://gitlab.kitware.com/help/api/lint.md
  * https://gitlab.kitware.com/help/api/version.md
  * https://gitlab.kitware.com/help/api/templates/dockerfiles.md
  * https://gitlab.kitware.com/help/api/templates/gitignores.md
  * https://gitlab.kitware.com/help/api/templates/gitlab_ci_ymls.md
  * https://gitlab.kitware.com/help/api/templates/licenses.md
