fn main() -> Result<(), strkey::Error> {
    let serialized = strkey::to_vec(&("account", 1234u32))?;

    println!("{}", String::from_utf8_lossy(&serialized));
    assert_eq!(&serialized, b"account:000004d2");

    let deserialized = strkey::from_slice::<(&str, u32)>(&serialized)?;

    assert_eq!(deserialized.0, "account");
    assert_eq!(deserialized.1, 1234);

    Ok(())
}
