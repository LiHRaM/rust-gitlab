extern crate ease;
use self::ease::Error as EaseError;
use self::ease::{Request, Response, Url};

extern crate serde_json;
use self::serde_json::Value;

extern crate url;
use self::url::ParseError as UrlError;
use self::url::percent_encoding::{PATH_SEGMENT_ENCODE_SET, percent_encode};

use std::borrow::Borrow;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    EaseError(EaseError),
    UrlError(UrlError),
    GitlabError(String),
}

impl Error {
    fn from_gitlab(value: Value) -> Self {
        let msg = value.pointer("/message")
                       .and_then(|s| s.as_string())
                       .unwrap_or_else(|| "unknown error");

        Error::GitlabError(msg.to_owned())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::EaseError(ref error)     => write!(f, "ease error: {:?}", error),
            Error::UrlError(ref error)      => write!(f, "url error: {}", error),
            Error::GitlabError(ref error)   => write!(f, "gitlab error: {}", error),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "GitLab API error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::EaseError(ref error) => Some(error),
            Error::UrlError(ref error)  => Some(error),
            _                           => None,
        }
    }
}

impl From<EaseError> for Error {
    fn from(error: EaseError) -> Self {
        Error::EaseError(error)
    }
}

impl From<UrlError> for Error {
    fn from(error: UrlError) -> Self {
        Error::UrlError(error)
    }
}

#[derive(Clone, Copy)]
pub enum CommitStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

impl Borrow<str> for CommitStatus {
    fn borrow(&self) -> &str {
        match *self {
            CommitStatus::Pending   => "pending",
            CommitStatus::Running   => "running",
            CommitStatus::Success   => "success",
            CommitStatus::Failed    => "failed",
            CommitStatus::Cancelled => "canceled",
        }
    }
}

pub struct Gitlab {
    base_url: Url,
    token: String,
}

header!{ (GitlabPrivateToken, "PRIVATE-TOKEN") => [String] }

pub type GitlabResult = Result<Value, Error>;

impl Gitlab {
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

    pub fn current_user(&self) -> GitlabResult {
        self._get("user")
    }

    pub fn user_by_name(&self, name: &str) -> GitlabResult {
        Self::_get_req(try!(self._mkrequest("users"))
                                .param("username", name))
    }

    pub fn user(&self, id: u64) -> GitlabResult {
        self._get(&format!("users/{}", id))
    }

    pub fn project_by_name(&self, name: &str) -> GitlabResult {
        self._get(&format!("projects/{}", percent_encode(name.as_bytes(), PATH_SEGMENT_ENCODE_SET)))
    }

    pub fn create_merge_request_note(&self, project: u64, id: u64, content: &str) -> GitlabResult {
        let path = &format!("projects/{}/merge_requests/{}/notes", project, id);

        Self::_post_req(try!(self._mkrequest(path))
                                 .param("body", content))
    }

    pub fn create_commit_status(&self, project: u64, sha: &str, state: CommitStatus, refname: &str, name: &str, description: &str) -> GitlabResult {
        let path = &format!("projects/{}/statuses/{}", project, sha);

        Self::_post_req(try!(self._mkrequest(path))
                                 .param("state", state.borrow())
                                 .param("ref", refname)
                                 .param("name", name)
                                 .param("description", description))
    }

    fn _mkrequest(&self, url: &str) -> Result<Request, Error> {
        let full_url = try!(self.base_url.join(url));
        let mut req = Request::new(full_url);

        req.header(GitlabPrivateToken(self.token.clone()));

        Ok(req)
    }

    fn _comm<F>(req: &mut Request, f: F) -> GitlabResult
        where F: FnOnce(&mut Request) -> Result<Response, EaseError> {
        match f(req) {
            Ok(rsp)     => rsp.from_json().map_err(|e| Error::EaseError(e)),
            Err(err)    => {
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
                Some(arr)   => arr,
                None        => return Err(Error::GitlabError("invalid page type".to_owned())),
            };

            results.extend(arr.into_iter().cloned());

            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            if arr.len() != per_page {
                break
            }
            page += 1;
        }

        Ok(Value::Array(results))
    }

    fn _get_paged(&self, url: &str) -> GitlabResult {
        Self::_get_paged_req(try!(self._mkrequest(url)))
    }
}
