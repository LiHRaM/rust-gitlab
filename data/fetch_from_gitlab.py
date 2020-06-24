#!/usr/bin/env python3

import json
import requests


def fetch_from_gitlab(token, endpoint, **kwargs):
    url = 'https://gitlab.kitware.com/api/v4' + endpoint
    response = requests.get(url, headers={'PRIVATE-TOKEN': token}, params=kwargs)
    return response.json()


def write_result(token, name, endpoint, dumpall=False):
    print('Writing out %s...' % name)
    result = fetch_from_gitlab(token, endpoint)
    if not dumpall:
        if type(result) == list:
            result = result[0]
        # Remove any keys from the result.
        result.pop('private_token', None)
        result.pop('runners_token', None)
        if type(result.get('identities')) == list:
            result['identities'] = []
        if type(result.get('current_sign_in_ip')) == str:
            result['current_sign_in_ip'] = "0.0.0.0"
        if type(result.get('last_sign_in_ip')) == str:
            result['last_sign_in_ip'] = "0.0.0.0"
    with open('%s.json' % name, 'w+') as fout:
        json.dump(result, fout, indent = 2, separators=(',', ': '), sort_keys=True)
        fout.write('\n')


REPO = 'utils%2Frust-gitlab'
USER = 11 # kwrobot
COMMIT = 'de4ac3cf96cb8a0893be22b03f5171d934f9d392'
ISSUE_ID = 6 # https://gitlab.kitware.com/utils/rust-gitlab/-/issues/6
MR_ID = 35 # https://gitlab.kitware.com/utils/rust-gitlab/-/merge_requests/35
MR_DISCUSSION_ID = 158 # https://gitlab.kitware.com/utils/rust-gitlab/-/merge_requests/35
NOTE_ID = 177359
PIPELINE_ID = 145400
PIPELINE2_ID = 168478
GROUP_ID = 498 # https://gitlab.kitware.com/utils


if __name__ == '__main__':
    import sys
    token = sys.argv[1]
    write_result(token, 'user_public', '/user')
    write_result(token, 'user', '/users/%d' % USER)
    write_result(token, 'project', '/projects/%s' % REPO)
    write_result(token, 'project_hook', '/projects/%s/hooks' % REPO)
    write_result(token, 'member', '/groups/utils/members')
    write_result(token, 'repo_branch', '/projects/%s/repository/branches/master' % REPO)
    write_result(token, 'repo_commit_detail', '/projects/%s/repository/commits/%s?stats=true' % (REPO, COMMIT))
    write_result(token, 'commit_note', '/projects/%s/repository/commits/%s/comments' % (REPO, COMMIT))
    write_result(token, 'commit_status', '/projects/%s/repository/commits/%s/statuses' % (REPO, COMMIT))
    write_result(token, 'issue', '/projects/%s/issues/%d' % (REPO, ISSUE_ID))
    write_result(token, 'merge_request', '/projects/%s/merge_requests/%d' % (REPO, MR_ID))
    write_result(token, 'issue_reference', '/projects/%s/merge_requests/%d/closes_issues' % (REPO, MR_ID))
    write_result(token, 'note', '/projects/%s/merge_requests/%d/notes' % (REPO, MR_ID))
    write_result(token, 'discussion', '/projects/%s/merge_requests/%d/discussions' % (REPO, MR_ID), dumpall=True)
    write_result(token, 'award_emoji', '/projects/%s/merge_requests/%d/notes/%d/award_emoji' % (REPO, MR_ID, NOTE_ID))
    write_result(token, 'resource_label_event', '/projects/%s/issues/%d/resource_label_events' % (REPO, ISSUE_ID))
    write_result(token, 'pipeline_basic', '/projects/%s/pipelines' % REPO)
    write_result(token, 'pipeline', '/projects/%s/pipelines/%d' % (REPO, PIPELINE_ID))
    write_result(token, 'group', '/groups/%s' % GROUP_ID)
    write_result(token, 'job', '/projects/%s/pipelines/%s/jobs' % (REPO, PIPELINE2_ID), dumpall=True)
    # FIXME: these are hidden behind a `403 forbidden`, so we use a hardcoded example instead.
    # write_result(token, 'pipeline_variable', '/projects/%s/pipelines/%d/variables' % (REPO, PIPELINE_ID))
