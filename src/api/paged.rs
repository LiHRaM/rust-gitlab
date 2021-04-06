// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use http::{header, HeaderMap, Request};
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

use crate::api::{query, ApiError, AsyncClient, AsyncQuery, Client, Endpoint, Query};

/// Errors which may occur with pagination.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PaginationError {
    /// A `Link` HTTP header can fail to parse.
    #[error("failed to parse a Link HTTP header: {}", source)]
    LinkHeader {
        /// The source of the error.
        #[from]
        source: LinkHeaderParseError,
    },
    /// An invalid URL can be returned.
    #[error("failed to parse a Link HTTP header URL: {}", source)]
    InvalidUrl {
        /// The source of the error.
        #[from]
        source: url::ParseError,
    },
}

#[derive(Debug)]
struct LinkHeader<'a> {
    url: &'a str,
    params: Vec<(&'a str, &'a str)>,
}

impl<'a> LinkHeader<'a> {
    fn parse(s: &'a str) -> Result<Self, LinkHeaderParseError> {
        let mut parts = s.split(';');

        let url_part = parts.next().expect("a split always has at least one part");
        let url = {
            let part = url_part.trim();
            if part.starts_with('<') && part.ends_with('>') {
                &part[1..part.len() - 1]
            } else {
                return Err(LinkHeaderParseError::NoBrackets);
            }
        };

        let params = parts
            .map(|part| {
                let part = part.trim();
                let mut halves = part.splitn(2, '=');
                let key = halves.next().expect("a split always has at least one part");
                let value = if let Some(value) = halves.next() {
                    if value.starts_with('"') && value.ends_with('"') {
                        &value[1..value.len() - 1]
                    } else {
                        value
                    }
                } else {
                    return Err(LinkHeaderParseError::MissingParamValue);
                };

                Ok((key, value))
            })
            .collect::<Result<Vec<_>, LinkHeaderParseError>>()?;

        Ok(Self {
            url,
            params,
        })
    }
}

/// An error which can occur when parsing a link header.
#[derive(Debug, Error)]
pub enum LinkHeaderParseError {
    /// An invalid HTTP header was found.
    #[error("invalid header")]
    InvalidHeader {
        /// The source of the error.
        #[from]
        source: reqwest::header::ToStrError,
    },
    /// The `url` for a `Link` header was missing `<>` brackets.
    #[error("missing brackets around url")]
    NoBrackets,
    /// A parameter for a `Link` header was missing a value.
    #[error("missing parameter value")]
    MissingParamValue,
}

impl LinkHeaderParseError {
    fn invalid_header(source: reqwest::header::ToStrError) -> Self {
        Self::InvalidHeader {
            source,
        }
    }
}

/// Pagination options for GitLab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pagination {
    /// Return all results.
    ///
    /// Note that some endpoints may have a server-side limit to the number of results (e.g.,
    /// `/projects` is limited to 10000 results).
    All,
    /// Limit to a number of results.
    Limit(usize),
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination::All
    }
}

const MAX_PAGE_SIZE: usize = 100;

impl Pagination {
    fn page_limit(self) -> usize {
        match self {
            Pagination::All => MAX_PAGE_SIZE,
            Pagination::Limit(size) => size.min(MAX_PAGE_SIZE),
        }
    }

    fn is_last_page<T>(self, last_page_size: usize, results: &[T]) -> bool {
        // If the last page has fewer elements than our limit, we're definitely done.
        if last_page_size < self.page_limit() {
            return true;
        }

        // Otherwise, check if we have results which fill our limit.
        if let Pagination::Limit(limit) = self {
            return limit <= results.len();
        }

        // We're not done yet.
        false
    }
}

/// A query modifier that paginates an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paged<E> {
    endpoint: E,
    pagination: Pagination,
}

/// Collect data from a paged endpoint.
pub fn paged<E>(endpoint: E, pagination: Pagination) -> Paged<E> {
    Paged {
        endpoint,
        pagination,
    }
}

/// A trait to indicate that an endpoint is pageable.
pub trait Pageable {
    /// Whether the endpoint uses keyset pagination or not.
    fn use_keyset_pagination(&self) -> bool {
        false
    }
}

impl<E, T, C> Query<Vec<T>, C> for Paged<E>
where
    E: Endpoint,
    E: Pageable,
    T: DeserializeOwned,
    C: Client,
{
    fn query(&self, client: &C) -> Result<Vec<T>, ApiError<C::Error>> {
        let url = {
            let mut url = client.rest_endpoint(&self.endpoint.endpoint())?;
            self.endpoint.parameters().add_to_url(&mut url);
            url
        };

        let mut page_num = 1;
        let per_page = self.pagination.page_limit();
        let per_page_str = format!("{}", per_page);

        let mut results = Vec::new();
        let mut next_url = None;
        let use_keyset_pagination = self.endpoint.use_keyset_pagination();

        let body = self.endpoint.body()?;

        loop {
            let page_url = if let Some(url) = next_url.take() {
                url
            } else {
                let page_str = format!("{}", page_num);
                let mut page_url = url.clone();

                {
                    let mut pairs = page_url.query_pairs_mut();
                    pairs.append_pair("per_page", &per_page_str);

                    if use_keyset_pagination {
                        pairs.append_pair("pagination", "keyset");
                    } else {
                        pairs.append_pair("page", &page_str);
                    }
                }

                page_url
            };

            let req = Request::builder()
                .method(self.endpoint.method())
                .uri(query::url_to_http_uri(page_url));
            let (req, data) = if let Some((mime, data)) = body.as_ref() {
                let req = req.header(header::CONTENT_TYPE, *mime);
                (req, data.clone())
            } else {
                (req, Vec::new())
            };
            let rsp = client.rest(req, data)?;
            let status = rsp.status();

            if use_keyset_pagination {
                next_url = next_page_from_headers(rsp.headers())?;
            }

            let v = serde_json::from_slice(rsp.body())?;
            if !status.is_success() {
                return Err(ApiError::from_gitlab(v));
            }

            let page =
                serde_json::from_value::<Vec<T>>(v).map_err(ApiError::data_type::<Vec<T>>)?;
            let page_len = page.len();
            results.extend(page);

            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            if self.pagination.is_last_page(page_len, &results) {
                break;
            }

            if use_keyset_pagination {
                if next_url.is_none() {
                    break;
                }
            } else {
                page_num += 1;
            }
        }

        Ok(results)
    }
}

#[async_trait]
impl<E, T, C> AsyncQuery<Vec<T>, C> for Paged<E>
where
    E: Endpoint + Sync,
    E: Pageable,
    T: DeserializeOwned + Send,
    C: AsyncClient + Sync,
{
    async fn query_async(&self, client: &C) -> Result<Vec<T>, ApiError<C::Error>> {
        let url = {
            let mut url = client.rest_endpoint(&self.endpoint.endpoint())?;
            self.endpoint.parameters().add_to_url(&mut url);
            url
        };

        let mut page_num = 1;
        let per_page = self.pagination.page_limit();
        let per_page_str = format!("{}", per_page);

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut next_url = None;
        let use_keyset_pagination = self.endpoint.use_keyset_pagination();

        let body = self.endpoint.body()?;

        loop {
            let page_url = if let Some(url) = next_url.take() {
                url
            } else {
                let page_str = format!("{}", page_num);
                let mut page_url = url.clone();

                {
                    let mut pairs = page_url.query_pairs_mut();
                    pairs.append_pair("per_page", &per_page_str);

                    if use_keyset_pagination {
                        pairs.append_pair("pagination", "keyset");
                    } else {
                        pairs.append_pair("page", &page_str);
                    }
                }

                page_url
            };

            let req = Request::builder()
                .method(self.endpoint.method())
                .uri(query::url_to_http_uri(page_url));
            let (req, data) = if let Some((mime, data)) = body.as_ref() {
                let req = req.header(header::CONTENT_TYPE, *mime);
                (req, data.clone())
            } else {
                (req, Vec::new())
            };
            let rsp = client.rest_async(req, data).await?;
            let status = rsp.status();

            if use_keyset_pagination {
                next_url = next_page_from_headers(rsp.headers())?;
            }

            let v = serde_json::from_slice(rsp.body())?;
            if !status.is_success() {
                return Err(ApiError::from_gitlab(v));
            }

            let page =
                serde_json::from_value::<Vec<T>>(v).map_err(ApiError::data_type::<Vec<T>>)?;
            let page_len = page.len();

            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            let is_last_page = {
                let mut locked_results = results.lock().expect("poisoned results");
                locked_results.extend(page);
                self.pagination.is_last_page(page_len, &locked_results)
            };
            if is_last_page {
                break;
            }

            if use_keyset_pagination {
                if next_url.is_none() {
                    break;
                }
            } else {
                page_num += 1;
            }
        }

        let mut locked_results = results.lock().expect("poisoned results");
        Ok(std::mem::take(&mut locked_results))
    }
}

fn next_page_from_headers(headers: &HeaderMap) -> Result<Option<Url>, PaginationError> {
    let link_headers = headers.get_all(reqwest::header::LINK).iter();
    // GitLab 14.0 will deprecate this header in preference for the W3C spec's `Link` header. Make
    // it less preferred to it in anticipation for this change.
    let links_headers = headers.get_all("Links").iter();
    link_headers
        .chain(links_headers)
        .map(|link| {
            let value = link
                .to_str()
                .map_err(LinkHeaderParseError::invalid_header)?;
            Ok(LinkHeader::parse(value)?)
        })
        .collect::<Result<Vec<_>, PaginationError>>()?
        .into_iter()
        .find_map(|header| {
            let is_next_link = header
                .params
                .into_iter()
                .any(|(key, value)| key == "rel" && value == "next");

            if is_next_link {
                Some(header.url.parse().map_err(PaginationError::from))
            } else {
                None
            }
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::api::endpoint_prelude::*;
    use crate::api::paged::LinkHeader;
    use crate::api::{self, ApiError, LinkHeaderParseError, Pagination, Query};
    use crate::test::client::{ExpectedUrl, PagedTestClient, SingleTestClient};

    #[test]
    fn test_link_header_no_brackets() {
        let err = LinkHeader::parse("url; param=value").unwrap_err();
        if let LinkHeaderParseError::NoBrackets = err {
            // expected error
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_link_header_no_param_value() {
        let err = LinkHeader::parse("<url>; param").unwrap_err();
        if let LinkHeaderParseError::MissingParamValue = err {
            // expected error
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_link_header_no_params() {
        let link = LinkHeader::parse("<url>").unwrap();
        assert_eq!(link.url, "url");
        assert_eq!(link.params.len(), 0);
    }

    #[test]
    fn test_link_header_quoted_params() {
        let link = LinkHeader::parse("<url>; param=\"value\"; param2=\"value\"").unwrap();
        assert_eq!(link.url, "url");
        assert_eq!(link.params.len(), 2);
        assert_eq!(link.params[0].0, "param");
        assert_eq!(link.params[0].1, "value");
        assert_eq!(link.params[1].0, "param2");
        assert_eq!(link.params[1].1, "value");
    }

    #[test]
    fn test_link_header_bare_params() {
        let link = LinkHeader::parse("<url>; param=value; param2=value").unwrap();
        assert_eq!(link.url, "url");
        assert_eq!(link.params.len(), 2);
        assert_eq!(link.params[0].0, "param");
        assert_eq!(link.params[0].1, "value");
        assert_eq!(link.params[1].0, "param2");
        assert_eq!(link.params[1].1, "value");
    }

    #[test]
    fn pagination_default() {
        assert_eq!(Pagination::default(), Pagination::All);
    }

    #[derive(Debug, Default)]
    struct Dummy {
        with_keyset: bool,
    }

    impl Endpoint for Dummy {
        fn method(&self) -> Method {
            Method::GET
        }

        fn endpoint(&self) -> Cow<'static, str> {
            "paged_dummy".into()
        }
    }

    impl Pageable for Dummy {
        fn use_keyset_pagination(&self) -> bool {
            self.with_keyset
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct DummyResult {
        value: u8,
    }

    #[test]
    fn test_gitlab_non_json_response() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .add_query_params(&[("page", "1"), ("per_page", "100")])
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "not json");
        let endpoint = Dummy::default();

        let res: Result<Vec<DummyResult>, _> = api::paged(endpoint, Pagination::All).query(&client);
        let err = res.unwrap_err();
        if let ApiError::Json {
            source,
        } = err
        {
            assert_eq!(format!("{}", source), "expected ident at line 1 column 2");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_gitlab_error_bad_json() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .add_query_params(&[("page", "1"), ("per_page", "100")])
            .status(StatusCode::NOT_FOUND)
            .build()
            .unwrap();
        let client = SingleTestClient::new_raw(endpoint, "");
        let endpoint = Dummy::default();

        let res: Result<Vec<DummyResult>, _> = api::paged(endpoint, Pagination::All).query(&client);
        let err = res.unwrap_err();
        if let ApiError::Json {
            source,
        } = err
        {
            assert_eq!(
                format!("{}", source),
                "EOF while parsing a value at line 1 column 0",
            );
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_gitlab_error_detection() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .add_query_params(&[("page", "1"), ("per_page", "100")])
            .status(StatusCode::NOT_FOUND)
            .build()
            .unwrap();
        let client = SingleTestClient::new_json(
            endpoint,
            &json!({
                "message": "dummy error message",
            }),
        );
        let endpoint = Dummy::default();

        let res: Result<Vec<DummyResult>, _> = api::paged(endpoint, Pagination::All).query(&client);
        let err = res.unwrap_err();
        if let ApiError::Gitlab {
            msg,
        } = err
        {
            assert_eq!(msg, "dummy error message");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_gitlab_error_detection_legacy() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .add_query_params(&[("page", "1"), ("per_page", "100")])
            .status(StatusCode::NOT_FOUND)
            .build()
            .unwrap();
        let client = SingleTestClient::new_json(
            endpoint,
            &json!({
                "error": "dummy error message",
            }),
        );
        let endpoint = Dummy::default();

        let res: Result<Vec<DummyResult>, _> = api::paged(endpoint, Pagination::All).query(&client);
        let err = res.unwrap_err();
        if let ApiError::Gitlab {
            msg,
        } = err
        {
            assert_eq!(msg, "dummy error message");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_gitlab_error_detection_unknown() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .add_query_params(&[("page", "1"), ("per_page", "100")])
            .status(StatusCode::NOT_FOUND)
            .build()
            .unwrap();
        let err_obj = json!({
            "bogus": "dummy error message",
        });
        let client = SingleTestClient::new_json(endpoint, &err_obj);
        let endpoint = Dummy::default();

        let res: Result<Vec<DummyResult>, _> = api::paged(endpoint, Pagination::All).query(&client);
        let err = res.unwrap_err();
        if let ApiError::GitlabUnrecognized {
            obj,
        } = err
        {
            assert_eq!(obj, err_obj);
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn test_pagination_limit() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .paginated(true)
            .build()
            .unwrap();
        let client = PagedTestClient::new_raw(
            endpoint,
            (0..=255).map(|value| {
                DummyResult {
                    value,
                }
            }),
        );
        let query = Dummy {
            with_keyset: false,
        };

        let res: Vec<DummyResult> = api::paged(query, Pagination::Limit(25))
            .query(&client)
            .unwrap();
        assert_eq!(res.len(), 25);
        for (i, value) in res.iter().enumerate() {
            assert_eq!(value.value, i as u8);
        }
    }

    #[test]
    fn test_pagination_all() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .paginated(true)
            .build()
            .unwrap();
        let client = PagedTestClient::new_raw(
            endpoint,
            (0..=255).map(|value| {
                DummyResult {
                    value,
                }
            }),
        );
        let query = Dummy::default();

        let res: Vec<DummyResult> = api::paged(query, Pagination::All).query(&client).unwrap();
        assert_eq!(res.len(), 256);
        for (i, value) in res.iter().enumerate() {
            assert_eq!(value.value, i as u8);
        }
    }

    #[test]
    fn test_keyset_pagination_limit() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .paginated(true)
            .build()
            .unwrap();
        let client = PagedTestClient::new_raw(
            endpoint,
            (0..=255).map(|value| {
                DummyResult {
                    value,
                }
            }),
        );
        let query = Dummy {
            with_keyset: true,
        };

        let res: Vec<DummyResult> = api::paged(query, Pagination::Limit(25))
            .query(&client)
            .unwrap();
        assert_eq!(res.len(), 25);
        for (i, value) in res.iter().enumerate() {
            assert_eq!(value.value, i as u8);
        }
    }

    #[test]
    fn test_keyset_pagination_all() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .paginated(true)
            .build()
            .unwrap();
        let client = PagedTestClient::new_raw(
            endpoint,
            (0..=255).map(|value| {
                DummyResult {
                    value,
                }
            }),
        );
        let query = Dummy {
            with_keyset: true,
        };

        let res: Vec<DummyResult> = api::paged(query, Pagination::All).query(&client).unwrap();
        assert_eq!(res.len(), 256);
        for (i, value) in res.iter().enumerate() {
            assert_eq!(value.value, i as u8);
        }
    }

    #[test]
    fn test_keyset_pagination_missing_header() {
        let endpoint = ExpectedUrl::builder()
            .endpoint("paged_dummy")
            .add_query_params(&[("pagination", "keyset"), ("per_page", "100")])
            .build()
            .unwrap();
        let data: Vec<_> = (0..=255)
            .map(|value| {
                DummyResult {
                    value,
                }
            })
            .collect();
        let client = SingleTestClient::new_raw(endpoint, serde_json::to_vec(&data).unwrap());
        let query = Dummy {
            with_keyset: true,
        };

        let res: Vec<DummyResult> = api::paged(query, Pagination::Limit(300))
            .query(&client)
            .unwrap();
        assert_eq!(res.len(), 256);
        for (i, value) in res.iter().enumerate() {
            assert_eq!(value.value, i as u8);
        }
    }
}
