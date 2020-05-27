# Endpoint status

This document categorizes the APIs as they pertain to this crate.

Last synced: 12.10.2

# Implemented

These API endpoints have been implemented.

  * `GET    /groups` `groups/groups.rs`
  * `POST   /groups` `groups/create.rs`
  * `GET    /groups/:group` `groups/group.rs`
  * `GET    /groups/:group/members` `groups/members/members.rs`
  * `POST   /groups/:group/members` `groups/members/add.rs`
  * `GET    /groups/:group/members/:id` `groups/members/member.rs`
  * `DELETE /groups/:group/members/:id` `groups/members/remove.rs`
  * `GET    /projects` `projects/projects.rs`
  * `POST   /projects` `projects/projects/create.rs`
  * `GET    /projects/:project` `projects/projects/project.rs`
  * `PUT    /projects/:project` `projects/projects/edit.rs`
  * `GET    /projects/:project/environments` `projects/environments/environments.rs`
  * `GET    /projects/:project/environments/:id` `projects/environments/environment.rs`
  * `GET    /projects/:project/hooks` `projects/hooks/hooks.rs`
  * `POST   /projects/:project/hooks` `projects/hooks/create.rs`
  * `GET    /projects/:project/hooks/:id` `projects/hooks/hook.rs`
  * `GET    /projects/:project/issues` `projects/issues/issues.rs`
  * `POST   /projects/:project/issues` `projects/issues/create.rs`
  * `GET    /projects/:project/issues/:issue` `projects/issues/issue.rs`
  * `PUT    /projects/:project/issues/:issue` `projects/issues/edit.rs`
  * `GET    /projects/:project/jobs` `projects/jobs/jobs.rs`
  * `GET    /projects/:project/jobs/:id` `projects/jobs/job.rs`
  * `POST   /projects/:project/jobs/:id/cancel` `projects/jobs/cancel.rs`
  * `POST   /projects/:project/jobs/:id/erase` `projects/jobs/erase.rs`
  * `POST   /projects/:project/jobs/:id/retry` `projects/jobs/retry.rs`
  * `POST   /projects/:project/jobs/:id/play` `projects/jobs/play.rs`
  * `GET    /projects/:project/jobs/:id/trace` `projects/jobs/trace.rs`
  * `GET    /projects/:project/labels` `projects/labels/label.rs`
  * `POST   /projects/:project/labels` `projects/labels/create.rs`
  * `GET    /projects/:project/labels/:label` `projects/labels/labels.rs`
  * `GET    /projects/:project/members` `projects/members/members.rs`
  * `POST   /projects/:project/members` `projects/members/add.rs`
  * `GET    /projects/:project/members/:id` `projects/members/member.rs`
  * `GET    /projects/:project/pipeline` `projects/pipelines/create.rs`
  * `GET    /projects/:project/pipelines` `projects/pipelines/pipelines.rs`
  * `GET    /projects/:project/pipelines/:pipeline` `projects/pipelines/pipeline.rs`
  * `DELETE /projects/:project/pipelines/:pipeline` `projects/pipelines/delete.rs`
  * `POST   /projects/:project/pipelines/:pipeline/cancel` `projects/pipelines/cancel.rs`
  * `GET    /projects/:project/pipelines/:pipeline/jobs` `projects/pipelines/jobs.rs`
  * `POST   /projects/:project/pipelines/:pipeline/retry` `projects/pipelines/retry.rs`
  * `GET    /projects/:project/pipelines/:pipeline/variables` `projects/pipelines/variables.rs`
  * `POST   /projects/:project/protected_branches` `projects/protected_branches/protect.rs`
  * `DELETE /projects/:project/protected_branches/*branch` `projects/protected_branches/unprotect.rs`
  * `GET    /projects/:project/repository/branches` `projects/repository/branches/branches.rs`
  * `POST   /projects/:project/repository/branches` `projects/repository/branches/create.rs`
  * `GET    /projects/:project/repository/branches/:branch` `projects/repository/branches/branch.rs`
  * `GET    /projects/:project/repository/commits/:sha` `projects/repository/commits/commit.rs`
  * `GET    /projects/:project/repository/commits/:sha/comments` `projects/repository/commits/comments.rs`
  * `POST   /projects/:project/repository/commits/:sha/comments` `projects/repository/commits/comment.rs`
  * `GET    /projects/:project/repository/commits/:sha/statuses` `projects/repository/commits/statuses.rs`
  * `POST   /projects/:project/repository/files/*file_path` `projects/repository/files/create.rs`
  * `GET    /projects/:project/statuses/:sha` `projects/repository/commits/create_status.rs`
    Arguably, this should be `POST /projects/:project/repository/commits/:sha/statuses`.
    https://gitlab.com/gitlab-org/gitlab/-/issues/217412
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

  * `GET    /issues` https://gitlab.kitware.com/help/api/issues.md#list-issues
  * `PUT    /groups/:group` https://gitlab.kitware.com/help/api/groups.md#update-group
  * `DELETE /groups/:group` https://gitlab.kitware.com/help/api/groups.md#remove-group
  * `GET    /groups/:group/hooks` https://gitlab.kitware.com/help/api/groups.md#list-group-hooks
  * `POST   /groups/:group/hooks` https://gitlab.kitware.com/help/api/groups.md#add-group-hook
  * `GET    /groups/:group/hooks/:id` https://gitlab.kitware.com/help/api/groups.md#get-group-hook
  * `PUT    /groups/:group/hooks/:id` https://gitlab.kitware.com/help/api/groups.md#edit-group-hook
  * `DELETE /groups/:group/hooks/:id` https://gitlab.kitware.com/help/api/groups.md#delete-group-hook
  * `GET    /groups/:group/issues` https://gitlab.kitware.com/help/api/issues.md#list-group-issues
  * `POST   /groups/:group/ldap_sync` https://gitlab.kitware.com/help/api/groups.md#sync-group-with-ldap-starter
  * `GET    /groups/:group/ldap_group_links` https://gitlab.kitware.com/help/api/groups.md#list-ldap-group-links-starter
  * `POST   /groups/:group/ldap_group_links` https://gitlab.kitware.com/help/api/groups.md#add-ldap-group-link-with-cn-or-filter-starter
  * `DELETE /groups/:group/ldap_group_links/:cn` https://gitlab.kitware.com/help/api/groups.md#delete-ldap-group-link-starter (deprecated)
  * `DELETE /groups/:group/ldap_group_links/:provider/:cn` https://gitlab.kitware.com/help/api/groups.md#delete-ldap-group-link-starter (deprecated)
  * `DELETE /groups/:group/ldap_group_links` https://gitlab.kitware.com/help/api/groups.md#delete-ldap-group-link-with-cn-or-filter-starter
  * `GET    /groups/:group/members/all` https://gitlab.kitware.com/help/api/members.md#list-all-members-of-a-group-or-project-including-inherited-members
  * `GET    /groups/:group/members/all/:id` https://gitlab.kitware.com/help/api/members.md#get-a-member-of-a-group-or-project-including-inherited-members
  * `PUT    /groups/:group/members/:id` https://gitlab.kitware.com/help/api/members.md#edit-a-member-of-a-group-or-project
  * `GET    /groups/:group/projects` https://gitlab.kitware.com/help/api/groups.md#list-a-groups-projects
  * `POST   /groups/:group/projects/:id` https://gitlab.kitware.com/help/api/groups.md#transfer-project-to-group
  * `POST   /groups/:group/restore` https://gitlab.kitware.com/help/api/groups.md#restore-group-marked-for-deletion-premium
  * `GET    /groups/:group/subgroups` https://gitlab.kitware.com/help/api/groups.md#list-a-groups-subgroups
  * `DELETE /projects/:project` https://gitlab.kitware.com/help/api/projects.md#remove-project
  * `POST   /projects/:project/archive` https://gitlab.kitware.com/help/api/projects.md#archive-a-project
  * `POST   /projects/:project/environments` https://gitlab.kitware.com/help/api/environments.md#create-a-new-environment
  * `PUT    /projects/:project/environments/:id` https://gitlab.kitware.com/help/api/environments.md#edit-an-existing-environment
  * `DELETE /projects/:project/environments/:id` https://gitlab.kitware.com/help/api/environments.md#delete-an-environment
  * `POST   /projects/:project/environments/:id/stop` https://gitlab.kitware.com/help/api/environments.md#stop-an-environment
  * `POST   /projects/:project/fork` https://gitlab.kitware.com/help/api/projects.md#fork-project
  * `DELETE /projects/:project/fork` https://gitlab.kitware.com/help/api/projects.md#delete-an-existing-forked-from-relationship
  * `POST   /projects/:project/fork/:from` https://gitlab.kitware.com/help/api/projects.md#create-a-forked-fromto-relation-between-existing-projects
  * `GET    /projects/:project/forks` https://gitlab.kitware.com/help/api/projects.md#list-forks-of-a-project
  * `PUT    /projects/:project/hooks/:id` https://gitlab.kitware.com/help/api/projects.md#edit-project-hook
  * `DELETE /projects/:project/hooks/:id` https://gitlab.kitware.com/help/api/projects.md#delete-project-hook
  * `POST   /projects/:project/housekeeping` https://gitlab.kitware.com/help/api/projects.md#start-the-housekeeping-task-for-a-project
  * `DELETE /projects/:project/issues/:issue` https://gitlab.kitware.com/help/api/issues.md#delete-an-issue
  * `POST   /projects/:project/issues/:issue/add_spent_time` https://gitlab.kitware.com/help/api/issues.md#add-spent-time-for-an-issue
  * `GET    /projects/:project/issues/:issue/closed_by` https://gitlab.kitware.com/help/api/issues.md#list-merge-requests-that-will-close-issue-on-merge
  * `POST   /projects/:project/issues/:issue/move` https://gitlab.kitware.com/help/api/issues.md#move-an-issue
  * `GET    /projects/:project/issues/:issue/participants` https://gitlab.kitware.com/help/api/issues.md#participants-on-issues
  * `GET    /projects/:project/issues/:issue/related_merge_requests` https://gitlab.kitware.com/help/api/issues.md#list-merge-requests-related-to-issue
  * `POST   /projects/:project/issues/:issue/reset_spent_time` https://gitlab.kitware.com/help/api/issues.md#reset-spent-time-for-an-issue
  * `POST   /projects/:project/issues/:issue/reset_time_estimate` https://gitlab.kitware.com/help/api/issues.md#reset-the-time-estimate-for-an-issue
  * `POST   /projects/:project/issues/:issue/subscribe` https://gitlab.kitware.com/help/api/issues.md#subscribe-to-an-issue
  * `POST   /projects/:project/issues/:issue/time_estimate` https://gitlab.kitware.com/help/api/issues.md#set-a-time-estimate-for-an-issue
  * `GET    /projects/:project/issues/:issue/time_stats` https://gitlab.kitware.com/help/api/issues.md#get-time-tracking-stats
  * `POST   /projects/:project/issues/:issue/todo` https://gitlab.kitware.com/help/api/issues.md#create-a-todo
  * `POST   /projects/:project/issues/:issue/unsubscribe` https://gitlab.kitware.com/help/api/issues.md#unsubscribe-from-an-issue
  * `GET    /projects/:project/issues/:issue/user_agent_detail` https://gitlab.kitware.com/help/api/issues.md#get-user-agent-details
  * `GET    /projects/:project/jobs/artifacts/:ref/download` https://gitlab.kitware.com/help/api/jobs.md#download-the-artifacts-archive
  * `GET    /projects/:project/jobs/artifacts/:ref/raw/*artifact_path` https://gitlab.kitware.com/help/api/jobs.md#download-a-single-artifact-file-from-specific-tag-or-branch
  * `GET    /projects/:project/jobs/:id/artifacts` https://gitlab.kitware.com/help/api/jobs.md#get-job-artifacts
  * `DELETE /projects/:project/jobs/:id/artifacts` https://gitlab.kitware.com/help/api/jobs.md#delete-artifacts
  * `POST   /projects/:project/jobs/:id/artifacts/keep` https://gitlab.kitware.com/help/api/jobs.md#keep-artifacts
  * `GET    /projects/:project/jobs/:id/artifacts/*artifact_path` https://gitlab.kitware.com/help/api/jobs.md#download-a-single-artifact-file-by-job-id
  * `PUT    /projects/:project/labels/:label` https://gitlab.kitware.com/help/api/labels.md#edit-an-existing-label
  * `DELETE /projects/:project/labels/:label` https://gitlab.kitware.com/help/api/labels.md#delete-a-label
  * `PUT    /projects/:project/labels/:label/promote` https://gitlab.kitware.com/help/api/labels.md#promote-a-project-label-to-a-group-label
  * `POST   /projects/:project/labels/:label/subscribe` https://gitlab.kitware.com/help/api/labels.md#subscribe-to-a-label
  * `POST   /projects/:project/labels/:label/unsubscribe` https://gitlab.kitware.com/help/api/labels.md#unsubscribe-from-a-label
  * `GET    /projects/:project/languages` https://gitlab.kitware.com/help/api/projects.md#languages
  * `GET    /projects/:project/members/all` https://gitlab.kitware.com/help/api/members.md#list-all-members-of-a-group-or-project-including-inherited-members
  * `GET    /projects/:project/members/all/:id` https://gitlab.kitware.com/help/api/members.md#get-a-member-of-a-group-or-project-including-inherited-members
  * `PUT    /projects/:project/members/:id` https://gitlab.kitware.com/help/api/members.md#edit-a-member-of-a-group-or-project
  * `DELETE /projects/:project/members/:id` https://gitlab.kitware.com/help/api/members.md#remove-a-member-from-a-group-or-project
  * `POST   /projects/:project/mirror/pull` https://gitlab.kitware.com/help/api/projects.md#start-the-pull-mirroring-process-for-a-project-starter
  * `GET    /projects/:project/protected_branches` https://gitlab.kitware.com/help/api/protected_branches.md#list-protected-branches
  * `GET    /projects/:project/protected_branches/:branch` https://gitlab.kitware.com/help/api/protected_branches.md#get-a-single-protected-branch-or-wildcard-protected-branch
  * `PATCH  /projects/:project/protected_branches/:branch` https://gitlab.kitware.com/help/api/protected_branches.md#require-code-owner-approvals-for-a-single-branch
  * `POST   /projects/:project/pipeline` https://gitlab.kitware.com/help/api/pipelines.md#create-a-new-pipeline
  * `GET    /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#get-project-push-rules
  * `POST   /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#add-project-push-rule
  * `PUT    /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#edit-project-push-rule
  * `DELETE /projects/:project/push_rule` https://gitlab.kitware.com/help/api/projects.md#delete-project-push-rule
  * `DELETE /projects/:project/repository/branches/:branch` https://gitlab.kitware.com/help/api/branches.md#delete-repository-branch
  * `GET    /projects/:project/repository/commits` https://gitlab.kitware.com/help/api/commits.md#list-repository-commits
  * `POST   /projects/:project/repository/commits` https://gitlab.kitware.com/help/api/commits.md#create-a-commit-with-multiple-files-and-actions
  * `POST   /projects/:project/repository/commits/:sha/cherry_pick` https://gitlab.kitware.com/help/api/commits.md#cherry-pick-a-commit
  * `GET    /projects/:project/repository/commits/:sha/diffs` https://gitlab.kitware.com/help/api/commits.md#get-the-diff-of-a-commit
  * `GET    /projects/:project/repository/commits/:sha/merge_requests` https://gitlab.kitware.com/help/api/commits.md#list-merge-requests-associated-with-a-commit
  * `GET    /projects/:project/repository/commits/:sha/refs` https://gitlab.kitware.com/help/api/commits.md#get-references-a-commit-is-pushed-to
  * `POST   /projects/:project/repository/commits/:sha/revert` https://gitlab.kitware.com/help/api/commits.md#revert-a-commit
  * `GET    /projects/:project/repository/commits/:sha/signature` https://gitlab.kitware.com/help/api/commits.md#get-gpg-signature-of-a-commit
  * `GET    /projects/:project/repository/files/*file_path` https://gitlab.kitware.com/help/api/repository_files.md#get-file-from-repository
  * `HEAD   /projects/:project/repository/files/*file_path` https://gitlab.kitware.com/help/api/repository_files.md#get-file-from-repository
  * `GET    /projects/:project/repository/files/*file_path/blame` https://gitlab.kitware.com/help/api/repository_files.md#get-file-blame-from-repository
  * `GET    /projects/:project/repository/files/*file_path/raw` https://gitlab.kitware.com/help/api/repository_files.md#get-raw-file-from-repository
  * `PUT    /projects/:project/repository/files/*file_path` https://gitlab.kitware.com/help/api/repository_files.md#update-existing-file-in-repository
  * `DELETE /projects/:project/repository/files/*file_path` https://gitlab.kitware.com/help/api/repository_files.md#delete-existing-file-in-repository
  * `DELETE /projects/:project/repository/merged_branches` https://gitlab.kitware.com/help/api/branches.md#delete-merged-branches
  * `POST   /projects/:project/restore` https://gitlab.kitware.com/help/api/projects.md#restore-project-marked-for-deletion-premium
  * `POST   /projects/:project/share` https://gitlab.kitware.com/help/api/projects.md#share-project-with-group
  * `DELETE /projects/:project/share/:group` https://gitlab.kitware.com/help/api/projects.md#delete-a-shared-project-link-within-a-group
  * `GET    /projects/:project/snapshot` https://gitlab.kitware.com/help/api/projects.md#download-snapshot-of-a-git-repository
  * `POST   /projects/:project/star` https://gitlab.kitware.com/help/api/projects.md#star-a-project
  * `GET    /projects/:project/starrers` https://gitlab.kitware.com/help/api/projects.md#list-starrers-of-a-project
  * `PUT    /projects/:project/transfer` https://gitlab.kitware.com/help/api/projects.md#transfer-a-project-to-a-new-namespace
  * `POST   /projects/:project/unarchive` https://gitlab.kitware.com/help/api/projects.md#unarchive-a-project
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
  * `GET    /user_counts` https://gitlab.kitware.com/help/api/users.md#user-counts

## Endpoint groups

These pages document other endpoints not mentioned above:

  * https://gitlab.kitware.com/help/api/access_requests.md
  * https://gitlab.kitware.com/help/api/award_emoji.md
  * https://gitlab.kitware.com/help/api/container_registry.md
  * https://gitlab.kitware.com/help/api/custom_attributes.md
  * https://gitlab.kitware.com/help/api/dependencies.md
  * https://gitlab.kitware.com/help/api/deploy_keys.md
  * https://gitlab.kitware.com/help/api/deployments.md
  * https://gitlab.kitware.com/help/api/discussions.md
  * https://gitlab.kitware.com/help/api/error_tracking.md
  * https://gitlab.kitware.com/help/api/events.md
  * https://gitlab.kitware.com/help/api/issues_statistics.md
  * https://gitlab.kitware.com/help/api/boards.md
  * https://gitlab.kitware.com/help/api/issue_links.md
  * https://gitlab.kitware.com/help/api/managed_licenses.md
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
  * https://gitlab.kitware.com/help/api/protected_tags.md
  * https://gitlab.kitware.com/help/api/releases/index.md
  * https://gitlab.kitware.com/help/api/releases/links.md
  * https://gitlab.kitware.com/help/api/remote_mirrors.md
  * https://gitlab.kitware.com/help/api/repositories.md
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
  * https://gitlab.kitware.com/help/api/epic_issues.md
  * https://gitlab.kitware.com/help/api/epic_links.md
  * https://gitlab.kitware.com/help/api/epics.md
  * https://gitlab.kitware.com/help/api/group_badges.md
  * https://gitlab.kitware.com/help/api/group_boards.md
  * https://gitlab.kitware.com/help/api/group_labels.md
  * https://gitlab.kitware.com/help/api/group_level_variables.md
  * https://gitlab.kitware.com/help/api/group_milestones.md
  * https://gitlab.kitware.com/help/api/issues.md
  * https://gitlab.kitware.com/help/api/members.md
  * https://gitlab.kitware.com/help/api/admin_sidekiq_queues.md
  * https://gitlab.kitware.com/help/api/appearance.md
  * https://gitlab.kitware.com/help/api/applications.md
  * https://gitlab.kitware.com/help/api/audit_events.md
  * https://gitlab.kitware.com/help/api/avatar.md
  * https://gitlab.kitware.com/help/api/broadcast_messages.md
  * https://gitlab.kitware.com/help/api/snippets.md
  * https://gitlab.kitware.com/help/api/features.md
  * https://gitlab.kitware.com/help/api/geo_nodes.md
  * https://gitlab.kitware.com/help/api/group_activity_analytics.md
  * https://gitlab.kitware.com/help/api/import.md
  * https://gitlab.kitware.com/help/api/keys.md
  * https://gitlab.kitware.com/help/api/license.md
  * https://gitlab.kitware.com/help/api/markdown.md
  * https://gitlab.kitware.com/help/api/namespaces.md
  * https://gitlab.kitware.com/help/api/projects.md
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
