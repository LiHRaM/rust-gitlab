extern crate ease;
use self::ease::Error as EaseError;
use self::ease::{Request, Response, Url};

extern crate serde;
use self::serde::Deserialize;

extern crate serde_json;
use self::serde_json::from_value;

extern crate url;
use self::url::percent_encoding::{PATH_SEGMENT_ENCODE_SET, percent_encode};

use super::error::Error;
use super::types::*;

use std::borrow::Borrow;

/// A representation of the Gitlab API for a single user.
///
/// Separate users should use separate instances of this.
pub struct Gitlab {
    base_url: Url,
    token: String,
    api_user: UserFull,
}

// The header Gitlab uses to authenticate the user.
header!{ (GitlabPrivateToken, "PRIVATE-TOKEN") => [String] }

/// A JSON value return from Gitlab.
pub type GitlabResult<T: Deserialize> = Result<T, Error>;

impl Gitlab {
    /// Create a new Gitlab API representation.
    ///
    /// Errors out if `token` is invalid.
    pub fn new(host: &str, token: &str) -> Result<Self, Error> {
        let base_url = try!(Url::parse(&format!("https://{}/api/v3/", host)));

        // Ensure the API is working.
        let mut user_req = try!(Self::_mkrequest1(&base_url, token, "user"));
        let api_user = try!(Self::_get_req(&mut user_req));

        let api = Gitlab {
            base_url: base_url,
            token: token.to_owned(),
            api_user: api_user,
        };

        Ok(api)
    }

    /// The user the API is acting as.
    pub fn current_user(&self) -> GitlabResult<UserFull> {
        self._get("user")
    }

    /// Find a user by username.
    pub fn user_by_name<T: UserResult>(&self, name: &str) -> GitlabResult<T> {
        Self::_get_req(try!(self._mkrequest("users")).param("username", name))
    }

    /// Find a user by id.
    pub fn user<T: UserResult>(&self, id: UserId) -> GitlabResult<T> {
        self._get(&format!("users/{}", id))
    }

    /// Find a project by name.
    pub fn project_by_name(&self, name: &str) -> GitlabResult<Project> {
        self._get(&format!("projects/{}",
                           percent_encode(name.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    /// Create a note on a merge request.
    pub fn create_merge_request_note(&self, project: u64, id: u64, content: &str) -> GitlabResult<()> {
        let path = &format!("projects/{}/merge_requests/{}/notes", project, id);

        Self::_post_req(try!(self._mkrequest(path)).param("body", content))
    }

    /// Create a status message for a commit.
    pub fn create_commit_status(&self, project: u64, sha: &str, state: StatusState, refname: &str,
                                name: &str, description: &str)
                                -> GitlabResult<()> {
        let path = &format!("projects/{}/statuses/{}", project, sha);

        Self::_post_req(try!(self._mkrequest(path))
            .param("state", state.borrow())
            .param("ref", refname)
            .param("name", name)
            .param("description", description))
    }

    // Create a request with the proper common metadata for authentication.
    //
    // This method exists because we want to store the current user in the structure, but we don't
    // have a `self` before we create the structure. Making it `Option<>` is a little silly and
    // refactoring this out is worth the cleaner API.
    fn _mkrequest1<'a>(base_url: &Url, token: &str, url: &str) -> GitlabResult<Request<'a>> {
        let full_url = try!(base_url.join(url));
        let mut req = Request::new(full_url);

        debug!(target: "gitlab", "api call {}", url);

        req.header(GitlabPrivateToken(token.to_owned()));

        Ok(req)
    }

    // Create a request with the proper common metadata for authentication.
    fn _mkrequest(&self, url: &str) -> GitlabResult<Request> {
        Self::_mkrequest1(&self.base_url, &self.token, url)
    }

    // Refactored code which talks to Gitlab and transforms error messages properly.
    fn _comm<F, T>(req: &mut Request, f: F) -> GitlabResult<T>
        where F: FnOnce(&mut Request) -> Result<Response, EaseError>,
              T: Deserialize
    {
        match f(req) {
            Ok(rsp) => {
                let v = try!(rsp.from_json().map_err(|e| Error::EaseError(e)));

                Ok(try!(from_value::<T>(v)))
            },
            Err(err) => {
                if let EaseError::UnsuccessfulResponse(rsp) = err {
                    Err(Error::from_gitlab(try!(rsp.from_json())))
                } else {
                    Err(Error::EaseError(err))
                }
            },
        }
    }

    fn _get_req<T: Deserialize>(req: &mut Request) -> GitlabResult<T> {
        Self::_comm(req, |req| req.get())
    }

    fn _get<T: Deserialize>(&self, url: &str) -> GitlabResult<T> {
        Self::_get_req(&mut try!(self._mkrequest(url)))
    }

    fn _post_req<T: Deserialize>(req: &mut Request) -> GitlabResult<T> {
        Self::_comm(req, |req| req.post())
    }

    fn _post<T: Deserialize>(&self, url: &str) -> GitlabResult<T> {
        Self::_post_req(&mut try!(self._mkrequest(url)))
    }

    fn _put<T: Deserialize>(&self, url: &str) -> GitlabResult<T> {
        Self::_comm(&mut try!(self._mkrequest(url)), |req| req.put())
    }

    fn _get_paged_req<T: Deserialize>(req: Request) -> GitlabResult<Vec<T>> {
        let mut page_num = 1;
        let per_page = 100;
        let per_page_str = &format!("{}", per_page);

        let mut results: Vec<T> = vec![];

        loop {
            let page_str = &format!("{}", page_num);
            let mut page_req = req.clone();
            page_req.param("page", &page_str)
                    .param("per_page", per_page_str);
            let page = try!(Self::_get_req::<Vec<T>>(&mut page_req));
            let page_len = page.len();

            results.extend(page.into_iter());

            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            if page_len != per_page {
                break;
            }
            page_num += 1;
        }

        Ok(results)
    }

    // Handle paginated queries. Returns all results.
    fn _get_paged<T: Deserialize>(&self, url: &str) -> GitlabResult<Vec<T>> {
        Self::_get_paged_req(try!(self._mkrequest(url)))
    }
}
