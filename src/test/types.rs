// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate chrono;
use self::chrono::{TimeZone, UTC};

extern crate serde;
use self::serde::Deserialize;

extern crate serde_json;
use self::serde_json::from_reader;

use super::super::types::*;

use std::fs::File;

fn read_test_file<T: Deserialize>(name: &str) -> T {
    let fin = File::open(format!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/{}.json"), name))
        .unwrap();

    from_reader::<File, T>(fin).unwrap()
}

#[test]
fn test_read_award_emoji() {
    let award_emoji: AwardEmoji = read_test_file("award_emoji");

    assert_eq!(award_emoji.id, AwardId::new(335));
    assert_eq!(award_emoji.name, "8ball");
    assert_eq!(award_emoji.user.username, "ben.boeckel");
    assert_eq!(award_emoji.user.web_url,
               "https://gitlab.kitware.com/ben.boeckel");
    assert_eq!(award_emoji.user.name, "Ben Boeckel");
    assert_eq!(award_emoji.user.state, UserState::Active);
    assert_eq!(award_emoji.user.avatar_url,
               "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
    assert_eq!(award_emoji.user.id, UserId::new(13));
    assert_eq!(award_emoji.created_at,
               UTC.ymd(2016, 12, 7)
                   .and_hms_milli(16, 23, 46, 742));
    assert_eq!(award_emoji.updated_at,
               UTC.ymd(2016, 12, 7)
                   .and_hms_milli(16, 23, 46, 742));
    assert_eq!(award_emoji.awardable_id(),
               AwardableId::Note(NoteId::new(177359)));
    assert_eq!(award_emoji.awardable_type, AwardableType::Note);
}

#[test]
fn test_read_commit_note() {
    let commit_note: CommitNote = read_test_file("commit_note");

    assert_eq!(commit_note.note, "Example commit note for data fetching.");
    assert_eq!(commit_note.path, None);
    assert_eq!(commit_note.line, None);
    assert_eq!(commit_note.line_type, None);
    assert_eq!(commit_note.author.username, "ben.boeckel");
    assert_eq!(commit_note.author.web_url,
               "https://gitlab.kitware.com/ben.boeckel");
    assert_eq!(commit_note.author.name, "Ben Boeckel");
    assert_eq!(commit_note.author.state, UserState::Active);
    assert_eq!(commit_note.author.avatar_url,
               "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
    assert_eq!(commit_note.author.id, UserId::new(13));
    assert_eq!(commit_note.created_at,
               UTC.ymd(2016, 12, 7)
                   .and_hms_milli(16, 28, 33, 966));
}

#[test]
fn test_read_commit_status() {
    let commit_status: CommitStatus = read_test_file("commit_status");

    assert_eq!(commit_status.id, CommitStatusId::new(931434));
    assert_eq!(commit_status.sha,
               ObjectId::new("de4ac3cf96cb8a0893be22b03f5171d934f9d392".to_string()));
    assert_eq!(commit_status.ref_, Some("master".to_string()));
    assert_eq!(commit_status.status, StatusState::Success);
    assert_eq!(commit_status.name, "rust-gitlab-megas-linux-debug");
    assert_eq!(commit_status.target_url,
               Some("https://buildbot.kitware.com/builders/rust-gitlab-megas-linux-debug/builds/41"
                   .to_string()));
    assert_eq!(commit_status.description, Some("expected".to_string()));
    assert_eq!(commit_status.created_at,
               UTC.ymd(2016, 11, 8)
                   .and_hms_milli(14, 35, 32, 627));
    assert_eq!(commit_status.started_at, None);
    assert_eq!(commit_status.finished_at,
               Some(UTC.ymd(2016, 11, 8)
                   .and_hms_milli(14, 35, 32, 629)));
    assert_eq!(commit_status.allow_failure, false);
    assert_eq!(commit_status.author.username, "buildbot");
    assert_eq!(commit_status.author.web_url,
               "https://gitlab.kitware.com/buildbot");
    assert_eq!(commit_status.author.name, "buildbot");
    assert_eq!(commit_status.author.state, UserState::Active);
    assert_eq!(commit_status.author.avatar_url,
               "https://gitlab.kitware.com/uploads/user/avatar/35/buildbot-logo.png");
    assert_eq!(commit_status.author.id, UserId::new(35));
}

#[test]
fn test_read_issue() {
    let issue: Issue = read_test_file("issue");

    assert_eq!(issue.id, IssueId::new(69328));
    assert_eq!(issue.iid, 6);
    assert_eq!(issue.project_id, ProjectId::new(855));
    assert_eq!(issue.title, "fix documentation warnings");
    assert_eq!(issue.description, "");
    assert_eq!(issue.state, IssueState::Opened);
    assert_eq!(issue.created_at,
               UTC.ymd(2016, 10, 30)
                   .and_hms_milli(18, 54, 28, 954));
    assert_eq!(issue.updated_at,
               UTC.ymd(2016, 10, 30)
                   .and_hms_milli(18, 54, 29, 242));
    assert!(issue.labels.is_empty());
    assert!(issue.milestone.is_none());
    assert_eq!(issue.author.username, "ben.boeckel");
    assert_eq!(issue.author.web_url,
               "https://gitlab.kitware.com/ben.boeckel");
    assert_eq!(issue.author.name, "Ben Boeckel");
    assert_eq!(issue.author.state, UserState::Active);
    assert_eq!(issue.author.avatar_url,
               "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
    assert_eq!(issue.author.id, UserId::new(13));
    if let Some(ref assignee) = issue.assignee {
        assert_eq!(assignee.username, "ben.boeckel");
        assert_eq!(assignee.web_url, "https://gitlab.kitware.com/ben.boeckel");
        assert_eq!(assignee.name, "Ben Boeckel");
        assert_eq!(assignee.state, UserState::Active);
        assert_eq!(assignee.avatar_url,
                  "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
        assert_eq!(assignee.id, UserId::new(13));
    } else {
        panic!("expected to have an assignee for the issue");
    }
    assert_eq!(issue.subscribed, true);
    assert_eq!(issue.user_notes_count, 0);
    assert_eq!(issue.upvotes, 0);
    assert_eq!(issue.downvotes, 0);
    assert_eq!(issue.due_date, None);
    assert_eq!(issue.confidential, false);
    assert_eq!(issue.web_url,
               "https://gitlab.kitware.com/utils/rust-gitlab/issues/6");
}

#[test]
fn test_read_issue_reference() {
    let issue_reference: IssueReference = read_test_file("issue_reference");

    if let IssueReference::Internal(issue) = issue_reference {
        assert_eq!(issue.id, IssueId::new(69075));
        assert_eq!(issue.iid, 5);
        assert_eq!(issue.project_id, ProjectId::new(855));
        assert_eq!(issue.title, "Add project hook APIs");
        assert_eq!(issue.description,
                   "The workflow currently requires that the robot be able to register itself as \
                    a webhook for new projects. An API needs added for this.\n\nCc: @brad.king");
        assert_eq!(issue.state, IssueState::Closed);
        assert_eq!(issue.created_at,
                   UTC.ymd(2016, 10, 4)
                       .and_hms_milli(18, 59, 37, 178));
        assert_eq!(issue.updated_at,
                   UTC.ymd(2016, 10, 4)
                       .and_hms_milli(20, 18, 57, 519));
        assert!(issue.labels.is_empty());
        assert!(issue.milestone.is_none());
        assert_eq!(issue.author.username, "ben.boeckel");
        assert_eq!(issue.author.web_url,
                   "https://gitlab.kitware.com/ben.boeckel");
        assert_eq!(issue.author.name, "Ben Boeckel");
        assert_eq!(issue.author.state, UserState::Active);
        assert_eq!(issue.author.avatar_url,
                   "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
        assert_eq!(issue.author.id, UserId::new(13));
        if let Some(ref assignee) = issue.assignee {
            assert_eq!(assignee.username, "ben.boeckel");
            assert_eq!(assignee.web_url, "https://gitlab.kitware.com/ben.boeckel");
            assert_eq!(assignee.name, "Ben Boeckel");
            assert_eq!(assignee.state, UserState::Active);
            assert_eq!(assignee.avatar_url,
                      "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
            assert_eq!(assignee.id, UserId::new(13));
        } else {
            panic!("expected to have an assignee for the issue");
        }
        assert_eq!(issue.subscribed, true);
        assert_eq!(issue.user_notes_count, 0);
        assert_eq!(issue.upvotes, 0);
        assert_eq!(issue.downvotes, 0);
        assert_eq!(issue.due_date, None);
        assert_eq!(issue.confidential, false);
        assert_eq!(issue.web_url,
                   "https://gitlab.kitware.com/utils/rust-gitlab/issues/5");
    } else {
        panic!("expected to have an internal issue reference");
    }
}

#[test]
fn test_read_member() {
    let member: Member = read_test_file("member");

    assert_eq!(member.username, "kwrobot");
    assert_eq!(member.name, "Kitware Robot");
    assert_eq!(member.id, UserId::new(11));
    assert_eq!(member.state, UserState::Active);
    assert_eq!(member.avatar_url,
               "https://secure.gravatar.com/avatar/9ddcd45fcb89d966aab95b1f1002f84c?s=80&d=identicon");
    assert_eq!(member.web_url, "https://gitlab.kitware.com/kwrobot");
    assert_eq!(member.access_level, 50);
    assert_eq!(member.expires_at, None);
}

#[test]
fn test_read_merge_request() {
    let merge_request: MergeRequest = read_test_file("merge_request");

    assert_eq!(merge_request.id, MergeRequestId::new(21211));
    assert_eq!(merge_request.iid, 52);
    assert_eq!(merge_request.project_id, ProjectId::new(855));
    assert_eq!(merge_request.title, "Migrate to reqwest");
    assert_eq!(merge_request.description,
               Some("Currently, we cannot use both `rust-gitlab` (depending on `hyper`'s SSL \
                     support) and `libgit2` in the same application, because they require \
                     different versions of `rust-openssl` (hyper: 0.7.x, libgit2: 0.9.x).\r\n(I \
                     want to use the both libraries in my GitLab helper \
                     application.)\r\n\r\nseanmonstar (hyper's author) recommends using reqwest \
                     for HTTP clients.\r\n\r\nhttps://github.\
                     com/hyperium/hyper/issues/907#issuecomment-255509020\r\n\r\n> reqwest: Be \
                     the convenient, higher level Client crate. This will release very soon, \
                     and depend on rust-native-tls. Everyone using hyper for a client should be \
                     able to easily switch the reqwest, getting better TLS support immediately, \
                     and a (as much as possible) not-breaking API, even as hyper v0.10 comes \
                     out with its async Client.\r\n\r\nthoughts?\r\n\r\n(This MR is based on \
                     !51 changes)"
                   .to_string()));
    assert_eq!(merge_request.state, MergeRequestState::Opened);
    assert_eq!(merge_request.created_at,
               UTC.ymd(2016, 12, 7)
                   .and_hms_milli(14, 43, 31, 653));
    assert_eq!(merge_request.updated_at,
               UTC.ymd(2016, 12, 7)
                   .and_hms_milli(15, 45, 22, 184));
    assert_eq!(merge_request.target_branch, "master");
    assert_eq!(merge_request.source_branch, "migrate-to-reqwest");
    assert_eq!(merge_request.upvotes, 0);
    assert_eq!(merge_request.downvotes, 0);
    assert_eq!(merge_request.author.username, "gifnksm");
    assert_eq!(merge_request.author.web_url,
               "https://gitlab.kitware.com/gifnksm");
    assert_eq!(merge_request.author.name, "NAKASHIMA, Makoto");
    assert_eq!(merge_request.author.state, UserState::Active);
    assert_eq!(merge_request.author.avatar_url,
               "https://secure.gravatar.com/avatar/4f544d7f9fc4ae2b04512317f1a06b6e?s=80&d=identicon");
    assert_eq!(merge_request.author.id, UserId::new(1489));
    assert!(merge_request.assignee.is_none());
    assert_eq!(merge_request.source_project_id, ProjectId::new(1154));
    assert_eq!(merge_request.target_project_id, ProjectId::new(855));
    assert!(merge_request.labels.is_empty());
    assert_eq!(merge_request.work_in_progress, false);
    assert!(merge_request.milestone.is_none());
    assert_eq!(merge_request.merge_when_build_succeeds, false);
    assert_eq!(merge_request.merge_status, MergeStatus::CanBeMerged);
    assert_eq!(merge_request.sha,
               ObjectId::new("f2784e0607d08e79b361ccf58a8379b04de2df35"));
    assert_eq!(merge_request.merge_commit_sha, None);
    assert_eq!(merge_request.subscribed, true);
    assert_eq!(merge_request.user_notes_count, 7);
    assert_eq!(merge_request.should_remove_source_branch, None);
    assert_eq!(merge_request.force_remove_source_branch, Some(true));
    assert_eq!(merge_request.web_url,
               "https://gitlab.kitware.com/utils/rust-gitlab/merge_requests/52");
}

#[test]
fn test_read_note() {
    let note: Note = read_test_file("note");

    assert_eq!(note.id, NoteId::new(177371));
    assert_eq!(note.body,
               "Mentioned in commit 47d475d8625424bd37efd27f7097354306842b93");
    assert_eq!(note.attachment, None);
    assert_eq!(note.author.username, "brad.king");
    assert_eq!(note.author.web_url, "https://gitlab.kitware.com/brad.king");
    assert_eq!(note.author.name, "Brad King");
    assert_eq!(note.author.state, UserState::Active);
    assert_eq!(note.author.avatar_url,
               "https://secure.gravatar.com/avatar/0617392a2f9fd505720d0c42cefc1a10?s=80&d=identicon");
    assert_eq!(note.author.id, UserId::new(10));
    assert_eq!(note.created_at,
               UTC.ymd(2016, 10, 4)
                   .and_hms_milli(20, 18, 57, 786));
    assert_eq!(note.updated_at,
               UTC.ymd(2016, 10, 4)
                   .and_hms_milli(20, 18, 57, 786));
    assert_eq!(note.system, true);
    assert_eq!(note.noteable_id(),
               Some(NoteableId::MergeRequest(MergeRequestId::new(20215))));
    assert_eq!(note.noteable_type, NoteType::MergeRequest);
}

#[test]
fn test_read_project() {
    let project: Project = read_test_file("project");

    assert_eq!(project.id, ProjectId::new(855));
    assert_eq!(project.description,
               Some("Rust library for communicating with a Gitlab instance.".to_string()));
    assert_eq!(project.default_branch, Some("master".to_string()));
    assert!(project.tag_list.is_empty());
    assert_eq!(project.public, true);
    assert_eq!(project.archived, false);
    assert_eq!(project.visibility_level, 20);
    assert_eq!(project.ssh_url_to_repo,
               "git@gitlab.kitware.com:utils/rust-gitlab.git");
    assert_eq!(project.http_url_to_repo,
               "https://gitlab.kitware.com/utils/rust-gitlab.git");
    assert_eq!(project.web_url,
               "https://gitlab.kitware.com/utils/rust-gitlab");
    assert!(project.owner.is_none());
    assert_eq!(project.name, "rust-gitlab");
    assert_eq!(project.name_with_namespace, "Utils / rust-gitlab");
    assert_eq!(project.path, "rust-gitlab");
    assert_eq!(project.path_with_namespace, "utils/rust-gitlab");
    assert_eq!(project.container_registry_enabled, Some(true));
    assert_eq!(project.created_at,
               UTC.ymd(2016, 6, 29)
                   .and_hms_milli(17, 35, 12, 495));
    assert_eq!(project.last_activity_at,
               UTC.ymd(2016, 12, 7)
                   .and_hms_milli(15, 45, 22, 450));
    assert_eq!(project.shared_runners_enabled, true);
    assert_eq!(project.lfs_enabled, true);
    assert_eq!(project.creator_id, UserId::new(13));
    assert_eq!(project.namespace.name, "Utils");
    assert_eq!(project.namespace.path, "utils");
    assert_eq!(project.namespace.description, "");
    assert_eq!(project.namespace.id, 498);
    assert_eq!(project.namespace.owner_id(),
               NamespaceId::Group(GroupId::new(498)));
    assert_eq!(project.namespace.created_at,
               UTC.ymd(2016, 2, 3)
                   .and_hms_milli(21, 26, 13, 133));
    assert_eq!(project.namespace.updated_at,
               UTC.ymd(2016, 2, 3)
                   .and_hms_milli(21, 27, 5, 284));
    assert_eq!(project.namespace.deleted_at, None);
    assert_eq!(project.namespace.visibility_level, 20);
    if let Some(ref avatar) = project.namespace.avatar {
        assert_eq!(avatar.url, None);
    } else {
        panic!("expected to have an avatar for the namespace");
    }
    assert_eq!(project.namespace.lfs_enabled, None);
    assert_eq!(project.namespace.request_access_enabled, true);
    assert_eq!(project.namespace.share_with_group_lock, false);
    assert!(project.forked_from_project.is_none());
    assert_eq!(project.avatar_url, None);
    assert_eq!(project.star_count, 0);
    assert_eq!(project.forks_count, 2);
    assert_eq!(project.open_issues_count, Some(3));
    assert_eq!(project.public_builds, true);
    assert!(project.shared_with_groups.is_empty());
    assert_eq!(project.only_allow_merge_if_build_succeeds, Some(false));
    assert_eq!(project.only_allow_merge_if_all_discussions_are_resolved,
               None);
    assert_eq!(project.request_access_enabled, true);
    assert_eq!(project.builds_enabled, false);
    assert_eq!(project.issues_enabled, true);
    assert_eq!(project.merge_requests_enabled, true);
    assert_eq!(project.snippets_enabled, false);
    assert_eq!(project.wiki_enabled, true);
    if let Some(ref permissions) = project.permissions {
        if let Some(ref group_access) = permissions.group_access {
            assert_eq!(group_access.access_level, 50);
            assert_eq!(group_access.notification_level, Some(3));
        } else {
            panic!("expected to have group access on the permissions");
        }
        assert!(permissions.project_access.is_none());
    } else {
        panic!("expected to have permissions available");
    }
}

#[test]
fn test_read_project_hook() {
    let project_hook: ProjectHook = read_test_file("project_hook");

    assert_eq!(project_hook.id, HookId::new(887));
    assert_eq!(project_hook.url, "http://kwrobot02:8080/event");
    assert_eq!(project_hook.created_at,
               UTC.ymd(2016, 6, 29)
                   .and_hms_milli(17, 35, 15, 771));
    assert_eq!(project_hook.push_events, true);
    assert_eq!(project_hook.tag_push_events, true);
    assert_eq!(project_hook.issues_events, true);
    assert_eq!(project_hook.merge_requests_events, true);
    assert_eq!(project_hook.note_events, true);
    assert_eq!(project_hook.enable_ssl_verification, false);
    assert_eq!(project_hook.build_events, false);
    assert_eq!(project_hook.pipeline_events, false);
    assert_eq!(project_hook.wiki_page_events, false);
}

#[test]
fn test_read_repo_branch() {
    let repo_branch: RepoBranch = read_test_file("repo_branch");

    assert_eq!(repo_branch.name, "master");
    if let Some(ref commit) = repo_branch.commit {
        assert_eq!(commit.author_email, "brad.king@kitware.com");
        assert_eq!(commit.author_name, "Brad King");
        assert_eq!(commit.authored_date,
                   UTC.ymd(2016, 11, 11)
                       .and_hms_milli(14, 59, 37, 0));
        assert_eq!(commit.committed_date,
                   UTC.ymd(2016, 11, 11)
                       .and_hms_milli(14, 59, 37, 0));
        assert_eq!(commit.committer_email, "kwrobot@kitware.com");
        assert_eq!(commit.committer_name, "Kitware Robot");
        assert_eq!(commit.id,
                   ObjectId::new("a418466df1c8b4e676e97d7d8d0d3cdfb1336558"));
        assert_eq!(commit.message,
                   "Merge topic 'nullable-fields'\n\n370267d4 types: more nullable \
                    fields\n\nAcked-by: Kitware Robot <kwrobot@kitware.com>\nReviewed-by: Brad \
                    King <brad.king@kitware.com>\nMerge-request: !47\n");
        assert_eq!(commit.parent_ids,
                   vec![
                        ObjectId::new("de4ac3cf96cb8a0893be22b03f5171d934f9d392"),
                        ObjectId::new("370267d429821e0f4354cb43c52ee2053c2cb744"),
                   ]);
    } else {
        panic!("expected to have a commit for the branch");
    }
    assert_eq!(repo_branch.merged, None);
    assert_eq!(repo_branch.protected, Some(true));
    assert_eq!(repo_branch.developers_can_push, Some(false));
    assert_eq!(repo_branch.developers_can_merge, Some(false));
}

#[test]
fn test_read_repo_commit_detail() {
    let repo_commit_detail: RepoCommitDetail = read_test_file("repo_commit_detail");

    assert_eq!(repo_commit_detail.id,
               ObjectId::new("de4ac3cf96cb8a0893be22b03f5171d934f9d392"));
    assert_eq!(repo_commit_detail.short_id, ObjectId::new("de4ac3cf"));
    assert_eq!(repo_commit_detail.title, "Merge topic 'mr-awards'");
    assert_eq!(repo_commit_detail.author_name, "Brad King");
    assert_eq!(repo_commit_detail.author_email, "brad.king@kitware.com");
    assert_eq!(repo_commit_detail.created_at,
               UTC.ymd(2016, 11, 8)
                   .and_hms_milli(14, 30, 13, 0));
    assert_eq!(repo_commit_detail.message,
               "Merge topic 'mr-awards'\n\na222c553 gitlab: add a method for MR award \
                queries\n\nAcked-by: Kitware Robot <kwrobot@kitware.com>\nReviewed-by: Brad King \
                <brad.king@kitware.com>\nMerge-request: !46\n");
    assert_eq!(repo_commit_detail.parent_ids,
               vec![
                   ObjectId::new("559f5f4a2bfe1f48e9e95afa09c029deb655cf7d"),
                   ObjectId::new("a222c5539569cda6999b8069f1e51a5202c30711"),
               ]);
    assert_eq!(repo_commit_detail.committed_date,
               UTC.ymd(2016, 11, 8)
                   .and_hms_milli(14, 30, 13, 0));
    assert_eq!(repo_commit_detail.authored_date,
               UTC.ymd(2016, 11, 8)
                   .and_hms_milli(14, 30, 13, 0));
    assert_eq!(repo_commit_detail.stats.additions, 8);
    assert_eq!(repo_commit_detail.stats.deletions, 0);
    assert_eq!(repo_commit_detail.stats.total, 8);
}

#[test]
fn test_read_user() {
    let user: User = read_test_file("user");

    assert_eq!(user.username, "kwrobot");
    assert_eq!(user.name, "Kitware Robot");
    assert_eq!(user.id, UserId::new(11));
    assert_eq!(user.state, UserState::Active);
    assert_eq!(user.avatar_url,
               "https://secure.gravatar.com/avatar/9ddcd45fcb89d966aab95b1f1002f84c?s=80&d=identicon");
    assert_eq!(user.web_url, "https://gitlab.kitware.com/kwrobot");
    assert_eq!(user.created_at,
               UTC.ymd(2015, 2, 26)
                   .and_hms_milli(15, 58, 34, 670));
    assert_eq!(user.is_admin, true);
    assert_eq!(user.bio, Some("".to_string()));
    assert_eq!(user.location, None);
    assert_eq!(user.skype, "");
    assert_eq!(user.linkedin, "");
    assert_eq!(user.twitter, "");
    assert_eq!(user.website_url, "");
    assert_eq!(user.organization, None);
}

#[test]
fn test_read_user_full() {
    let user_full: UserFull = read_test_file("user_full");

    assert_eq!(user_full.username, "ben.boeckel");
    assert_eq!(user_full.name, "Ben Boeckel");
    assert_eq!(user_full.id, UserId::new(13));
    assert_eq!(user_full.state, UserState::Active);
    assert_eq!(user_full.avatar_url,
               "https://secure.gravatar.com/avatar/2f5f7e99190174edb5a2f66b8653b0b2?s=80&d=identicon");
    assert_eq!(user_full.web_url, "https://gitlab.kitware.com/ben.boeckel");
    assert_eq!(user_full.created_at,
               UTC.ymd(2015, 2, 26)
                   .and_hms_milli(17, 23, 28, 730));
    assert_eq!(user_full.is_admin, false);
    assert_eq!(user_full.bio, None);
    assert_eq!(user_full.location, None);
    assert_eq!(user_full.skype, "");
    assert_eq!(user_full.linkedin, "");
    assert_eq!(user_full.twitter, "");
    assert_eq!(user_full.website_url, "");
    assert_eq!(user_full.organization, None);
    assert_eq!(user_full.last_sign_in_at,
               Some(UTC.ymd(2016, 12, 7)
                   .and_hms_milli(15, 5, 56, 167)));
    assert_eq!(user_full.confirmed_at,
               UTC.ymd(2015, 2, 26)
                   .and_hms_milli(17, 23, 28, 693));
    assert_eq!(user_full.email, "ben.boeckel@kitware.com");
    assert_eq!(user_full.theme_id, ThemeId::new(2));
    assert_eq!(user_full.color_scheme_id, ColorSchemeId::new(2));
    assert_eq!(user_full.projects_limit, 50);
    assert_eq!(user_full.current_sign_in_at,
               Some(UTC.ymd(2016, 12, 7)
                   .and_hms_milli(16, 15, 50, 720)));
    assert!(user_full.identities.is_empty());
    assert_eq!(user_full.can_create_group, true);
    assert_eq!(user_full.can_create_project, true);
    assert_eq!(user_full.two_factor_enabled, true);
    assert_eq!(user_full.external, false);
}
