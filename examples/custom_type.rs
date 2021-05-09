use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AccountId {
    domain: String,
    user_id: u32,
}

fn main() -> Result<(), strkey::Error> {
    let serialized = strkey::to_vec(&(
        "account",
        AccountId {
            domain: "abc".to_string(),
            user_id: 1234,
        },
    ))?;

    println!("{}", String::from_utf8_lossy(&serialized));
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

    Ok(())
}
