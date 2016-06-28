extern crate chrono;
use self::chrono::{TimeZone, UTC};

extern crate serde;
use self::serde::Deserialize;

extern crate serde_json;
use self::serde_json::from_reader;

use super::types::*;

use std::fs::File;

fn read_test_file<T: Deserialize>(name: &str) -> T {
    let fin = File::open(format!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/{}.json"), name)).unwrap();

    from_reader::<File, T>(fin).unwrap()
}

#[test]
fn test_read_user_basic() {
    let user_basic: UserBasic = read_test_file("user_basic");

    assert_eq!(user_basic.username, "ben.boeckel");
    assert_eq!(user_basic.name, "Ben Boeckel");
    assert_eq!(user_basic.id, UserId::new(13));
    assert_eq!(user_basic.state, UserState::Active);
    assert_eq!(user_basic.avatar_url, "https://example.com/avatar.png");
    assert_eq!(user_basic.web_url, "https://gitlab.example.com/u/ben.boeckel");
}

#[test]
fn test_read_user() {
    let user: User = read_test_file("user");

    assert_eq!(user.username, "ben.boeckel");
    assert_eq!(user.name, "Ben Boeckel");
    assert_eq!(user.id, UserId::new(13));
    assert_eq!(user.state, UserState::Active);
    assert_eq!(user.avatar_url, "https://example.com/avatar.png");
    assert_eq!(user.web_url, "https://gitlab.example.com/u/ben.boeckel");
    assert_eq!(user.created_at, UTC.ymd(2015, 2, 26)
                                   .and_hms_milli(17, 23, 28, 730));
    assert_eq!(user.is_admin, false);
    assert_eq!(user.bio, None);
    assert_eq!(user.location, None);
    assert_eq!(user.skype, "");
    assert_eq!(user.linkedin, "");
    assert_eq!(user.twitter, "");
    assert_eq!(user.website_url, "");
}
