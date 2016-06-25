extern crate ease;
use self::ease::Error as EaseError;
use self::ease::{Request, Response, Url};

extern crate serde_json;
use self::serde_json::Value;

extern crate url;
use self::url::percent_encoding::{PATH_SEGMENT_ENCODE_SET, percent_encode};

use super::error::Error;

use std::borrow::Borrow;

#[derive(Clone, Copy)]
/// States for commit statuses.
pub enum CommitStatus {
    /// The check is queued.
    Pending,
    /// The check is currently running.
    Running,
    /// The check succeeded.
    Success,
    /// The check failed.
    Failed,
    /// The check was cancelled.
    Cancelled,
}

impl Borrow<str> for CommitStatus {
    fn borrow(&self) -> &str {
        match *self {
            CommitStatus::Pending => "pending",
            CommitStatus::Running => "running",
            CommitStatus::Success => "success",
            CommitStatus::Failed => "failed",
            CommitStatus::Cancelled => "canceled", // [sic]
        }
    }
}

/// A representation of the Gitlab API for a single user.
///
/// Separate users should use separate instances of this.
pub struct Gitlab {
    base_url: Url,
    token: String,
}

// The header Gitlab uses to authenticate the user.
header!{ (GitlabPrivateToken, "PRIVATE-TOKEN") => [String] }

/// A JSON value return from Gitlab.
pub type GitlabResult = Result<Value, Error>;

impl Gitlab {
    /// Create a new Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new(host: &str, token: &str) -> Result<Self, Error> {
        let api = Gitlab {
            base_url: try!(Url::parse(&format!("https://{}/api/v3/", host))),
            token: token.to_owned(),
        };

        // Ensure the API is working.
        try!(api.current_user());

        // TODO: store user information.

        Ok(api)
    }

    /// The user the API is acting as.
    pub fn current_user(&self) -> GitlabResult {
        self._get("user")
    }

    /// Find a user by username.
    pub fn user_by_name(&self, name: &str) -> GitlabResult {
        Self::_get_req(try!(self._mkrequest("users")).param("username", name))
    }

    /// Find a user by id.
    pub fn user(&self, id: u64) -> GitlabResult {
        self._get(&format!("users/{}", id))
    }

    /// Find a project by name.
    pub fn project_by_name(&self, name: &str) -> GitlabResult {
        self._get(&format!("projects/{}",
                           percent_encode(name.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    /// Create a note on a merge request.
    pub fn create_merge_request_note(&self, project: u64, id: u64, content: &str) -> GitlabResult {
        let path = &format!("projects/{}/merge_requests/{}/notes", project, id);

        Self::_post_req(try!(self._mkrequest(path)).param("body", content))
    }

    /// Create a status message for a commit.
    pub fn create_commit_status(&self, project: u64, sha: &str, state: CommitStatus, refname: &str,
                                name: &str, description: &str)
                                -> GitlabResult {
        let path = &format!("projects/{}/statuses/{}", project, sha);

        Self::_post_req(try!(self._mkrequest(path))
            .param("state", state.borrow())
            .param("ref", refname)
            .param("name", name)
            .param("description", description))
    }

    // Create a request with the proper common metadata for authentication.
    fn _mkrequest(&self, url: &str) -> Result<Request, Error> {
        let full_url = try!(self.base_url.join(url));
        let mut req = Request::new(full_url);

        debug!(target: "gitlab", "api call {}", url);

        req.header(GitlabPrivateToken(self.token.clone()));

        Ok(req)
    }

    // Refactored code which talks to Gitlab and transforms error messages properly.
    fn _comm<F>(req: &mut Request, f: F) -> GitlabResult
        where F: FnOnce(&mut Request) -> Result<Response, EaseError>
    {
        match f(req) {
            Ok(rsp) => rsp.from_json().map_err(|e| Error::EaseError(e)),
            Err(err) => {
                if let EaseError::UnsuccessfulResponse(rsp) = err {
                    Err(Error::from_gitlab(try!(rsp.from_json())))
                } else {
                    Err(Error::EaseError(err))
                }
            },
        }
    }

    fn _get_req(req: &mut Request) -> GitlabResult {
        Self::_comm(req, |req| req.get())
    }

    fn _get(&self, url: &str) -> GitlabResult {
        Self::_get_req(&mut try!(self._mkrequest(url)))
    }

    fn _post_req(req: &mut Request) -> GitlabResult {
        Self::_comm(req, |req| req.post())
    }

    fn _post(&self, url: &str) -> GitlabResult {
        Self::_post_req(&mut try!(self._mkrequest(url)))
    }

    fn _put(&self, url: &str) -> GitlabResult {
        Self::_comm(&mut try!(self._mkrequest(url)), |req| req.put())
    }

    fn _get_paged_req(req: Request) -> GitlabResult {
        let mut page = 1;
        let per_page = 100;
        let per_page_str = &format!("{}", per_page);

        let mut results: Vec<Value> = vec![];

        loop {
            let page_str = &format!("{}", page);
            let mut page_req = req.clone();
            let res = try!(Self::_get_req(page_req.param("page", &page_str)
                                                  .param("per_page", per_page_str)));

            let arr = match res.as_array() {
                Some(arr) => arr,
                None => return Err(Error::GitlabError("invalid page type".to_owned())),
            };

            results.extend(arr.into_iter().cloned());

            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            if arr.len() != per_page {
                break;
            }
            page += 1;
        }

        Ok(Value::Array(results))
    }

    // Handle paginated queries. Returns all results.
    fn _get_paged(&self, url: &str) -> GitlabResult {
        Self::_get_paged_req(try!(self._mkrequest(url)))
    }
}
