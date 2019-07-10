# Gitlab API

This library implements an interface to communicate with a Gitlab instance. Not
all API endpoints are implemented, but patches are welcome.

The API is based off of the GitLab 11.10 API v4 and will likely aggressively track
new API additions, so the newest release may not support talking to older
releases where fields have been added.

All API types should be implemented in the [types](src/types.rs) module. These
types should generally be implemented based on the `lib/api/entities.rb`
module in the Gitlab repository. However, in the interest of usability,
entities may be combined using `Option` to handle the differences. Generally,
this should be done where the difference is "small". As a concrete example,
the `Project` entity has dozens of fields and `ProjectWithAccess` has one
additional field (`permissions`) which is added using `Option` rather than
creating a new `ProjectWithAccess` structure which only differs in this field.

In short, map the API as close as possible, but also know when to bend the
rules.

If you run into places where Gitlab dumps a JSON value rather than an actual
entity, please consider updating upstream to use a real entity so that changes
to the structure are easier to track.

# Versioning

Since this crate follows Gitlab upstream, semantic versioning may not be
possible. Instead, the crate uses the following versioning scheme:

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
gitlab = "=0.1110.0"
```

# API bugs

Sometimes, the API will return `null` for fields that have been added after the
entry was created. In these cases, mark the field as an `Option` with a comment
describing why it is so.
