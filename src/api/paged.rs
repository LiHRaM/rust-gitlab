// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use reqwest::header::{self, HeaderMap};
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

use crate::api::{ApiError, Client, Endpoint, Query};

/// Errors which may occur with pagination.
#[derive(Debug, Error)]
// TODO #[non_exhaustive]
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
    /// This is here to force `_` matching right now.
    ///
    /// **DO NOT USE**
    #[doc(hidden)]
    #[error("unreachable...")]
    _NonExhaustive,
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

            let req = client.build_rest(self.endpoint.method(), page_url);
            let req = if let Some((mime, data)) = body.as_ref() {
                req.header(header::CONTENT_TYPE, *mime).body(data.clone())
            } else {
                req
            };
            let rsp = client.rest(req)?;
            let status = rsp.status();

            if use_keyset_pagination {
                next_url = next_page_from_headers(rsp.headers())?;
            }

            let v = serde_json::from_reader(rsp)?;
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

fn next_page_from_headers(headers: &HeaderMap) -> Result<Option<Url>, PaginationError> {
    headers
        .get_all(reqwest::header::LINK)
        .iter()
        .map(|link| {
            let value = link
                .to_str()
                .map_err(LinkHeaderParseError::invalid_header)?;
            Ok(LinkHeader::parse(value)?)
        })
        .collect::<Result<Vec<_>, PaginationError>>()?
        .into_iter()
        .filter_map(|header| {
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
        .next()
        .transpose()
}

#[cfg(test)]
mod tests {
    use crate::api::paged::LinkHeader;
    use crate::api::{LinkHeaderParseError, Pagination};

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
}
