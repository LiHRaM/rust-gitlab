// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::{self, NameOrId};
use crate::api::endpoint_prelude::*;

/// Create a new webhook for a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct CreateHook<'a> {
    /// The project to create a webhook within.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    /// The URL for the webhook to contact.
    #[builder(setter(into))]
    url: Cow<'a, str>,

    /// Whether to send push events for this webhook or not.
    #[builder(default)]
    push_events: Option<bool>,
    /// Filter which branches send push events for this webhook.
    #[builder(setter(into), default)]
    push_events_branch_filter: Option<Cow<'a, str>>,
    /// Whether to send issue events for this webhook or not.
    #[builder(default)]
    issues_events: Option<bool>,
    /// Whether to send confidential issue events for this webhook or not.
    #[builder(default)]
    confidential_issues_events: Option<bool>,
    /// Whether to send merge request events for this webhook or not.
    #[builder(default)]
    merge_requests_events: Option<bool>,
    /// Whether to send tag events for this webhook or not.
    #[builder(default)]
    tag_push_events: Option<bool>,
    /// Whether to send note (comment) events for this webhook or not.
    #[builder(default)]
    note_events: Option<bool>,
    /// Whether to send job events for this webhook or not.
    #[builder(default)]
    job_events: Option<bool>,
    /// Whether to send pipeline events for this webhook or not.
    #[builder(default)]
    pipeline_events: Option<bool>,
    /// Whether to send wiki page events for this webhook or not.
    #[builder(default)]
    wiki_page_events: Option<bool>,

    /// Whether to verify SSL/TLS certificates for the webhook endpoint or not.
    #[builder(default)]
    enable_ssl_verification: Option<bool>,
    /// A secret token to include in webhook deliveries.
    ///
    /// This may be used to ensure that the webhook is actually coming from the GitLab instance.
    #[builder(setter(into), default)]
    token: Option<Cow<'a, str>>,
}

impl<'a> CreateHook<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> CreateHookBuilder<'a> {
        CreateHookBuilder::default()
    }
}

impl<'a> Endpoint for CreateHook<'a> {
    fn method(&self) -> Method {
        Method::POST
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/hooks", self.project).into()
    }

    fn add_parameters(&self, mut pairs: Pairs) {
        pairs.append_pair("url", &self.url);

        self.push_events
            .map(|value| pairs.append_pair("push_events", common::bool_str(value)));
        self.push_events_branch_filter
            .as_ref()
            .map(|value| pairs.append_pair("push_events_branch_filter", value));
        self.issues_events
            .map(|value| pairs.append_pair("issues_events", common::bool_str(value)));
        self.confidential_issues_events
            .map(|value| pairs.append_pair("confidential_issues_events", common::bool_str(value)));
        self.merge_requests_events
            .map(|value| pairs.append_pair("merge_requests_events", common::bool_str(value)));
        self.tag_push_events
            .map(|value| pairs.append_pair("tag_push_events", common::bool_str(value)));
        self.note_events
            .map(|value| pairs.append_pair("note_events", common::bool_str(value)));
        self.job_events
            .map(|value| pairs.append_pair("job_events", common::bool_str(value)));
        self.pipeline_events
            .map(|value| pairs.append_pair("pipeline_events", common::bool_str(value)));
        self.wiki_page_events
            .map(|value| pairs.append_pair("wiki_page_events", common::bool_str(value)));

        self.enable_ssl_verification
            .map(|value| pairs.append_pair("enable_ssl_verification", common::bool_str(value)));
        self.token
            .as_ref()
            .map(|value| pairs.append_pair("token", value));
    }
}

#[cfg(test)]
mod tests {
    use crate::api::projects::hooks::CreateHook;

    #[test]
    fn project_and_url_are_necessary() {
        let err = CreateHook::builder().build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn project_is_necessary() {
        let err = CreateHook::builder().url("url").build().unwrap_err();
        assert_eq!(err, "`project` must be initialized");
    }

    #[test]
    fn url_is_necessary() {
        let err = CreateHook::builder()
            .project("project")
            .build()
            .unwrap_err();
        assert_eq!(err, "`url` must be initialized");
    }

    #[test]
    fn project_and_url_are_sufficient() {
        CreateHook::builder()
            .project("project")
            .url("url")
            .build()
            .unwrap();
    }
}
