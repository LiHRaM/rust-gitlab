# Gitlab API

This library implements an interface to communicate with a Gitlab instance. Not
all API endpoints are implemented, but patches are welcome.

The API is based off of the GitLab 13.1 API v4 and will likely aggressively track
new API additions, so not all available parameters or types will support
arbitrarily old GitLab instances (usually query parameters will be ignored and
type fields cause deserialization errors).

The endpoints that are supported all live under the [`api`](src/api.rs) module.
Each endpoint may be constructed using a "builder" pattern to provide supported
fields. To use an endpoint, you may query it using the
[`Query`](src/api/query.rs) trait. There are additional helpers to handle
different cases:

  - [`api::ignore`](src/api/ignore.rs): Ignore the GitLab response (useful for
    `POST` or `PUT` endpoints).
  - [`api::paged`](src/api/paged.rs): Fetch results that are paginated.
  - [`api::raw`](src/api/raw.rs): Return the raw data from GitLab instead of
    deserializing into a structure.
  - [`api::sudo`](src/api/sudo.rs): Modify an endpoint using GitLab's `sudo`
    parameter for masquerading as another user (requires an administrator
    token).

All endpoints return data types of the caller's choosing that implement
`serde`'s `Deserialize` trait. Callers should define their own structures for
obtaining data from the API. This allows the structure to be more easily
changeable for different GitLab versions (rather than this crate being pinned
to a given version).

# Versioning

Since this crate follows Gitlab upstream, semantic versioning may not be
possible. Instead, the crate uses the following versioning scheme:

  * Gitlab 13.1 support → 0.1301.x
  * Gitlab 13.0 support → 0.1300.x
  * Gitlab 12.10 support → 0.1210.x
  * Gitlab 12.9 support → 0.1209.x
  * Gitlab 12.8 support → 0.1208.x
  * Gitlab 12.7 support → 0.1207.x
  * Gitlab 12.6 support → 0.1206.x
  * Gitlab 12.5 support → 0.1205.x
  * Gitlab 12.4 support → 0.1204.x
  * Gitlab 12.3 support → 0.1203.x
  * Gitlab 12.2 support → 0.1202.x
  * Gitlab 12.1 support → 0.1201.x
  * Gitlab 12.0 support → 0.1200.x
  * Gitlab 11.11 support → 0.1111.x
  * Gitlab 11.10 support → 0.1110.x
  * Gitlab 11.9 support → 0.1109.x
  * Gitlab 11.8 support → 0.1108.x
  * Gitlab 11.7 support → 0.1107.x
  * Gitlab 11.6 support → 0.1106.x
  * Gitlab 11.5 support → 0.1105.x
  * Gitlab 11.4 support → 0.1104.x
  * Gitlab 11.3 support → 0.1103.x
  * Gitlab 11.2 support → 0.1102.x
  * Gitlab 11.1 support → 0.1101.x
  * Gitlab 11.0 support → 0.1100.x
  * Gitlab 10.8 support → 0.1008.x
  * Gitlab 10.7 support → 0.1007.x
  * Gitlab 10.6 support → 0.1006.x
  * Gitlab 10.5 support → 0.1005.x
  * Gitlab 10.4 support → 0.1004.x
  * Gitlab 10.3 support → 0.1003.x
  * Gitlab 10.2 support → 0.1002.x
  * Gitlab 10.1 support → 0.1001.x
  * Gitlab 10.0 support → 0.1000.x
  * Gitlab 9.5 support → 0.905.x
  * Gitlab 9.4 support → 0.904.x
  * Gitlab 9.3 support → 0.903.x
  * Gitlab 9.2 support → 0.902.x
  * Gitlab 9.1 support → 0.901.x
  * Gitlab 9.0 support → 0.900.x
  * Gitlab 8.17 support → 0.817.x
  * Gitlab 8.16 support → 0.816.x

Minor versions may fix bugs, add API endpoint bindings, or improve webhook
coverage. It is recommended to depend on the full version of the crate since
types may change in patch-level updates in order to match Gitlab's interface:

```toml
gitlab = "=0.1301.0"
```

# API bugs

Sometimes, the API will return `null` for fields that have been added after the
entry was created. In these cases, mark the field as an `Option` with a comment
describing why it is so.
