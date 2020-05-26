// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use crate::api::common::NameOrId;
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

    fn body(&self) -> Result<Option<(&'static str, Vec<u8>)>, BodyError> {
        let mut params = FormParams::default();

        params
            .push("url", &self.url)
            .push_opt("push_events", self.push_events)
            .push_opt(
                "push_events_branch_filter",
                self.push_events_branch_filter.as_ref(),
            )
            .push_opt("issues_events", self.issues_events)
            .push_opt(
                "confidential_issues_events",
                self.confidential_issues_events,
            )
            .push_opt("merge_requests_events", self.merge_requests_events)
            .push_opt("tag_push_events", self.tag_push_events)
            .push_opt("note_events", self.note_events)
            .push_opt("job_events", self.job_events)
            .push_opt("pipeline_events", self.pipeline_events)
            .push_opt("wiki_page_events", self.wiki_page_events)
            .push_opt("enable_ssl_verification", self.enable_ssl_verification)
            .push_opt("token", self.token.as_ref());

        params.into_body()
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
