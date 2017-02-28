# Gitlab API

This library implements an interface to communicate with a Gitlab instance. Not
all API endpoints are implemented, but patches are welcome.

The API is based off of the 8.16.0 API and will likely aggressively track new
API additions, so the newest release may not support talking to older releases
where fields have been added.

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

# API bugs

Sometimes, the API will return `null` for fields that have been added after the
entry was created. In these cases, mark the field as an `Option` with a comment
describing why it is so.
