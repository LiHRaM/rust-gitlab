// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use thiserror::Error;
use url::form_urlencoded::Serializer;
use url::{Url, UrlQuery};

use crate::gitlab::{Gitlab, GitlabError, PaginationError};

pub type Pairs<'a> = Serializer<'a, UrlQuery<'a>>;

const MAX_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pagination {
    All,
    Limit(usize),
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination::All
    }
}

impl Pagination {
    fn page_limit(self) -> usize {
        match self {
            Pagination::All => MAX_PAGE_SIZE,
            Pagination::Limit(size) => size.min(MAX_PAGE_SIZE),
        }
    }

    fn is_last_page<T>(self, last_page_size: usize, results: &[T]) -> bool {
        // If the last page has fewer elements than our limit, we're definitely done.
        if last_page_size <= self.page_limit() {
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

pub trait Query<T> {
    fn query(&self, client: &Gitlab) -> Result<T, GitlabError>;
}

pub trait SingleQuery<T>
where
    T: DeserializeOwned,
{
    type FormData: Serialize;

    fn method(&self) -> Method;
    fn endpoint(&self) -> String;

    fn add_parameters(&self, pairs: Pairs);
    fn form_data(&self) -> Self::FormData;

    fn single_query(&self, client: &Gitlab) -> Result<T, GitlabError> {
        let mut url = client.rest_endpoint(self.endpoint())?;
        self.add_parameters(url.query_pairs_mut());

        let req = client
            .build_rest(self.method(), url)
            .form(&self.form_data());
        let rsp = client.rest(req)?;
        let status = rsp.status();
        let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
        if !status.is_success() {
            return Err(GitlabError::from_gitlab(v));
        }

        serde_json::from_value::<T>(v).map_err(GitlabError::data_type::<T>)
    }

    fn no_answer_query(&self, client: &Gitlab) -> Result<(), GitlabError> {
        let mut url = client.rest_endpoint(self.endpoint())?;
        self.add_parameters(url.query_pairs_mut());

        let req = client
            .build_rest(self.method(), url)
            .form(&self.form_data());
        let rsp = client.rest(req)?;
        if !rsp.status().is_success() {
            let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
            return Err(GitlabError::from_gitlab(v));
        }

        Ok(())
    }
}

struct LinkHeader<'a> {
    url: &'a str,
    params: Vec<(&'a str, &'a str)>,
}

impl<'a> LinkHeader<'a> {
    fn parse(s: &'a str) -> Result<Self, LinkHeaderParseError> {
        let mut parts = s.split(';');

        let url = if let Some(part) = parts.next() {
            let part = part.trim();
            if part.starts_with('<') && part.ends_with('>') {
                &part[1..part.len() - 1]
            } else {
                return Err(LinkHeaderParseError::NoBrackets);
            }
        } else {
            return Err(LinkHeaderParseError::MissingUrl);
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

#[derive(Debug, Error)]
pub enum LinkHeaderParseError {
    #[error("invalid header")]
    InvalidHeader {
        #[from]
        source: reqwest::header::ToStrError,
    },
    #[error("missing url")]
    MissingUrl,
    #[error("missing brackets around url")]
    NoBrackets,
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

pub trait PagedQuery<T, F>: SingleQuery<Vec<T>, FormData = F>
where
    T: DeserializeOwned,
    F: Serialize,
{
    fn pagination(&self) -> Pagination {
        Pagination::All
    }

    fn use_keyset_pagination(&self) -> bool {
        false
    }

    fn paged_query(&self, client: &Gitlab) -> Result<Vec<T>, GitlabError> {
        let url = {
            let mut url = client.rest_endpoint(self.endpoint())?;
            self.add_parameters(url.query_pairs_mut());
            url
        };

        let pagination = self.pagination();
        let mut page_num = 1;
        let per_page = pagination.page_limit();
        let per_page_str = format!("{}", per_page);

        let mut results = Vec::new();
        let mut next_url = None;
        let use_keyset_pagination = self.use_keyset_pagination();

        loop {
            let page_url = if let Some(url) = next_url.take() {
                url
            } else {
                let page_str = format!("{}", page_num);
                let mut page_url = url.clone();
                page_url
                    .query_pairs_mut()
                    .extend_pairs(&[("page", &page_str), ("per_page", &per_page_str)]);

                if use_keyset_pagination {
                    page_url
                        .query_pairs_mut()
                        .append_pair("pagination", "keyset");
                }

                page_url
            };

            let req = client.build_rest(Method::GET, page_url);
            let rsp = client.rest(req)?;
            let status = rsp.status();

            if use_keyset_pagination {
                next_url = next_page_from_headers(rsp.headers())?;
            }

            let v = serde_json::from_reader(rsp).map_err(GitlabError::json)?;
            if !status.is_success() {
                return Err(GitlabError::from_gitlab(v));
            }

            let page =
                serde_json::from_value::<Vec<T>>(v).map_err(GitlabError::data_type::<Vec<T>>)?;
            let page_len = page.len();
            results.extend(page);

            // Gitlab used to have issues returning paginated results; these have been fixed since,
            // but if it is needed, the bug manifests as Gitlab returning *all* results instead of
            // just the requested results. This can cause an infinite loop here if the number of
            // total results is exactly equal to `per_page`.
            if pagination.is_last_page(page_len, &results) {
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
