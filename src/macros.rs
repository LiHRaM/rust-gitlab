// Copyright 2016 Kitware, Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

macro_rules! impl_id {
    ( $name:ident ) => {
        /* This bit of the macro handles the repetitive nature of creating new identifiers.
         * Unfortunately, it doesn't work with serde_codegen, so until either plugins are stable or
         * nightly is required, just implement things by hand.
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(u64);
         */

        impl $name {
            /// Create a new id.
            pub fn new(id: u64) -> Self {
                $name(id)
            }

            /// The value of the id.
            pub fn value(&self) -> u64 {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

macro_rules! enum_serialize {
    ( $name:ident -> $desc:expr, $( $value:ident => $str:expr ),+ ) => {
        impl Borrow<str> for $name {
            fn borrow(&self) -> &str {
                match *self {
                    $( $name::$value => $str, )*
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
                serializer.serialize_str(self.borrow())
            }
        }

        impl Deserialize for $name {
            fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
                let val = try!(String::deserialize(deserializer));

                match val.borrow() {
                    $( $str => Ok($name::$value), )*
                    v => {
                        error!(target: "gitlab", concat!("unknown ", $desc, " from gitlab: {}"), v);
                        Err(D::Error::invalid_value(concat!("invalid ", $desc)))
                    },
                }
            }
        }
    };
}
