# strkey

strkey is Rust library for serialization of values into an human-readable encoding that preserves lexicographic sort order. The encoded format is useful for key-value stores/databases that maintains keys in sorted order.

[![Crates.io](https://img.shields.io/crates/v/strkey)](https://crates.io/crates/strkey) [![docs.rs](https://img.shields.io/docsrs/strkey)](https://docs.rs/strkey)

This crate is similar to [bytekey](https://crates.io/crates/bytekey)/[bytekey-fix](https://crates.io/crates/bytekey-fix)/[bytekey2](https://crates.io/crates/bytekey2) except those this crate encodes into a human-readable, UTF-8 encoded string.

Most data types in the [Serde data model](https://serde.rs/data-model.html) are supported. The encoding consists of each encoded value separated by a deliminator (a colon `:` by default). Integers, floats, and byte arrays are encoded in hexadecimal while strings are left as is in UTF-8. Full details are listed in the documentation in the `ser` module. Note that the encoding is not self-describing, that is, types are not encoded within the format.

## Quick start

Remember to add `strkey` to your Cargo.toml first.

```rust
let serialized = strkey::to_vec(&("account", 1234u32))?;

assert_eq!(&serialized, b"account:000004d2");

let deserialized = strkey::from_slice::<(&str, u32)>(&serialized)?;

assert_eq!(deserialized.0, "account");
assert_eq!(deserialized.1, 1234);
```

More complicated schemas can be modelled: (Enable the [`derive` feature](https://serde.rs/derive.html) in serde if needed.)

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AccountId {
    domain: String,
    user_id: u32,
}

let serialized = strkey::to_vec(&(
    "account",
    AccountId {
        domain: "abc".to_string(),
        user_id: 1234,
    },
))?;

assert_eq!(&serialized, b"account:abc:000004d2");

let deserialized = strkey::from_slice::<(String, AccountId)>(&serialized)?;

assert_eq!(&deserialized.0, "account");
assert_eq!(
    &deserialized.1,
    &AccountId {
        domain: "abc".to_string(),
        user_id: 1234
    }
);
```

## Contributing

If you have any issues or features, please use the GitHub Issues and Pull Request sections.

## License

Copyright 2021 Christopher Foo. Licensed under MPL-2.0.
