#!/usr/bin/env python

import json
import requests


def fetch_from_gitlab(token, endpoint, **kwargs):
    url = 'https://gitlab.kitware.com/api/v3' + endpoint
    response = requests.get(url, headers={'PRIVATE-TOKEN': token}, params=kwargs)
    return response.json()


def write_result(token, name, endpoint):
    print('Writing out %s...' % name)
    result = fetch_from_gitlab(token, endpoint)
    if type(result) == list:
        result = result[0]
    # Remove any keys from the result.
    result.pop('private_token', None)
    result.pop('runners_token', None)
    with open('%s.json' % name, 'w+') as fout:
        json.dump(result, fout)
        fout.write('\n')


REPO = 'utils%2Frust-gitlab'
USER = 11 # kwrobot
COMMIT = 'de4ac3cf96cb8a0893be22b03f5171d934f9d392'
ISSUE_ID = 69328 # https://gitlab.kitware.com/utils/rust-gitlab/issues/6
MR_ID = 20215 # https://gitlab.kitware.com/utils/rust-gitlab/merge_requests/35
NOTE_ID = 177359


if __name__ == '__main__':
    import sys
    token = sys.argv[1]
    write_result(token, 'user_full', '/user')
    write_result(token, 'user', '/users/%d' % USER)
    write_result(token, 'project', '/projects/%s' % REPO)
    write_result(token, 'project_hook', '/projects/%s/hooks' % REPO)
    write_result(token, 'member', '/groups/utils/members')
    write_result(token, 'repo_branch', '/projects/%s/repository/branches/master' % REPO)
    write_result(token, 'repo_commit_detail', '/projects/%s/repository/commits/%s' % (REPO, COMMIT))
    write_result(token, 'commit_note', '/projects/%s/repository/commits/%s/comments' % (REPO, COMMIT))
    write_result(token, 'commit_status', '/projects/%s/repository/commits/%s/statuses' % (REPO, COMMIT))
    write_result(token, 'issue', '/projects/%s/issues/%d' % (REPO, ISSUE_ID))
    write_result(token, 'merge_request', '/projects/%s/merge_requests/%d' % (REPO, MR_ID))
    write_result(token, 'issue_reference', '/projects/%s/merge_requests/%d/closes_issues' % (REPO, MR_ID))
    write_result(token, 'note', '/projects/%s/merge_requests/%d/notes' % (REPO, MR_ID))
    write_result(token, 'award_emoji', '/projects/%s/merge_requests/%d/notes/%d/award_emoji' % (REPO, MR_ID, NOTE_ID))
