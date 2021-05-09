# strkey

strkey is Rust library for serialization of values into an human-readable encoding that preserves lexicographic sort order. The encoded format is useful for key-value stores/databases that maintains keys in sorted order.

[![Crates.io](https://img.shields.io/crates/v/strkey)](https://crates.io/crates/strkey) [![docs.rs](https://img.shields.io/docsrs/strkey)](https://docs.rs/strkey)

This crate is similar to [bytekey](https://crates.io/crates/bytekey)/[bytekey-fix](https://crates.io/crates/bytekey-fix)/[bytekey2](https://crates.io/crates/bytekey2) except those this crate encodes into a human-readable, UTF-8 encoded string.

Most data types in the [Serde data model](https://serde.rs/data-model.html) are supported. The encoding consists of each encoded value separated by a deliminator (a colon `:` by default). Note that the encoding is not self-describing:

* For unit type, it's not considered a value and no encoding action happens.
* For booleans, they are encoded as literals "true" or "false".
* For integers, they are encoded as fixed-width hexadecimal of their big-endian representations. Signed integers are preprocessed with some bit manipulation, as in the bytekey crate, so that negative numbers sort first.
* For floating point numbers, they're preprocessed with some bit manipulation, as in the bytekey crate, so that negative numbers sort first. Then encoded as hexadecimal.
* For strings, no special encoding is done since they are already UTF-8 encoded.
* For byte arrays (requires [serde_bytes](https://crates.io/crates/serde_bytes)), they are encoded as hexadecimal.
* For tuples, each encoded value is separated by the configured deliminator. Note that deliminator are emitted along values; the data structure itself doesn't cause deliminators to be emitted.
* For structs, the field names are *not* encoded. Only the values are encoded as it were a tuple. This can be useful for labeling each part of the database key without encoding the schema itself.
* For enums with unit variants, only the name of the enum's variant is encoded. The name of the enum itself is not encoded.
* For option, maps, sequences, and enums with tuple or struct variants are not supported and return an error.

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
