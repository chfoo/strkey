//! Serialization
//!
//! ## Details
//!
//! Most data types in the [Serde data model](https://serde.rs/data-model.html) are supported. The encoding consists of each encoded value separated by a deliminator (a colon `:` by default). Note that the encoding is not self-describing:
//!
//! * For unit type, it's not considered a value and no encoding action happens.
//! * For booleans, they are encoded as literals "true" or "false".
//! * For integers, they are encoded as fixed-width hexadecimal of their big-endian representations. Signed integers are preprocessed with some bit manipulation, as in the bytekey crate, so that negative numbers sort first.
//! * For floating point numbers, they're preprocessed with some bit manipulation, as in the bytekey crate, so that negative numbers sort first. Then encoded as hexadecimal.
//! * For strings, no special encoding is done since they are already UTF-8 encoded.
//! * For byte arrays (requires [serde_bytes](https://crates.io/crates/serde_bytes)), they are encoded as hexadecimal.
//! * For tuples, each encoded value is separated by the configured deliminator. Note that deliminator are emitted along values; the data structure itself doesn't cause deliminators to be emitted.
//! * For structs, the field names are *not* encoded. Only the values are encoded as it were a tuple. This can be useful for labeling each part of the database key without encoding the schema itself.
//! * For enums with unit variants, only the name of the enum's variant is encoded. The name of the enum itself is not encoded.
//! * For option, maps, sequences, and enums with tuple or struct variants are not supported and return an error.
use std::io::Write;

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::error::Error;

/// Serializer for encoding values into strkey encoding.
///
/// Example:
///
/// ```
/// use serde::ser::Serialize;
/// # use strkey::Serializer;
///
/// # fn main() -> Result<(), strkey::Error> {
/// let mut buffer = Vec::new();
/// let mut serializer = Serializer::new(&mut buffer);
/// ("account", 1234u32).serialize(&mut serializer)?;
/// assert_eq!(&buffer, b"account:000004d2");
/// # Ok(())
/// # }
/// ```
pub struct Serializer<W: Write> {
    output: W,
    deliminator: String,
    first_part_written: bool,
    buffer: Vec<u8>,
}

impl<W: Write> Serializer<W> {
    /// Serialize the value into the given writer using the default options.
    pub fn new(writer: W) -> Self {
        Self {
            output: writer,
            deliminator: ":".to_string(),
            first_part_written: false,
            buffer: Vec::new(),
        }
    }

    /// Unwrap and return the wrapped writer.
    pub fn into_inner(self) -> W {
        self.output
    }

    /// Returns the deliminator used to separate values.
    pub fn deliminator(&self) -> &str {
        &self.deliminator
    }

    /// Sets the deliminator used to separate values.
    pub fn set_deliminator<S: Into<String>>(&mut self, deliminator: S) {
        self.deliminator = deliminator.into();
    }

    /// Sets the deliminator used to separate values and returns a new serializer.
    pub fn with_deliminator<S: Into<String>>(mut self, deliminator: S) -> Self {
        self.set_deliminator(deliminator);
        self
    }

    fn maybe_write_deliminator(&mut self) -> Result<(), Error> {
        if self.first_part_written {
            self.output.write_all(&self.deliminator.as_bytes())?;
        } else {
            self.first_part_written = true;
        }
        Ok(())
    }

    fn write_encode_hex(&mut self, data: &[u8]) -> Result<(), Error> {
        self.buffer.resize(data.len() * 2, 0);
        hex::encode_to_slice(data, &mut self.buffer).unwrap();
        self.output.write_all(&self.buffer)?;
        Ok(())
    }
}

impl<'a, W: Write> serde::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        self.output.write_all(if v { b"true" } else { b"false" })?;
        Ok(())
    }

    // signed integer magic https://github.com/danburkert/bytekey/blob/6980b9e33281d875f03f4c9a953b93a384eac085/src/encoder.rs#L322

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = (v ^ i8::MIN).to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = (v ^ i16::MIN).to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = (v ^ i32::MIN).to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = (v ^ i64::MIN).to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = (v ^ i128::MIN).to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = v.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = v.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = v.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = v.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let buf = v.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    // floating point magic https://github.com/danburkert/bytekey/blob/6980b9e33281d875f03f4c9a953b93a384eac085/src/encoder.rs#L340

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let val = v.to_bits() as i32;
        let t = (val >> 31) | i32::MIN;
        let val = val ^ t;
        let buf = val.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let val = v.to_bits() as i64;
        let t = (val >> 63) | i64::MIN;
        let val = val ^ t;
        let buf = val.to_be_bytes();
        self.write_encode_hex(&buf)?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        let mut buf = [0u8; 4];
        self.output.write_all(v.encode_utf8(&mut buf).as_bytes())?;

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        self.output.write_all(v.as_bytes())?;

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        self.write_encode_hex(v)?;

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.maybe_write_deliminator()?;

        self.output.write_all(variant.as_bytes())?;

        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::UnsupportedType)
    }
}

impl<'a, W: Write> SerializeSeq for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeMap for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeStructVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

/// Serializes the given value to a vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer).with_deliminator(":");
    value.serialize(&mut serializer)?;

    Ok(buffer)
}

/// Serializes the given value to the writer.
pub fn to_writer<W, T>(mut writer: W, value: &T) -> Result<(), Error>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = Serializer::new(&mut writer).with_deliminator(":");
    value.serialize(&mut serializer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Serialize;
    use serde_bytes::{ByteBuf, Bytes};

    use super::*;

    #[test]
    fn test_bool() {
        let key = to_vec(&true).unwrap();
        assert_eq!(&key, b"true");

        let key = to_vec(&false).unwrap();
        assert_eq!(&key, b"false");
    }

    #[test]
    fn test_unsigned_integer() {
        let key = to_vec(&0xaau8).unwrap();
        assert_eq!(&key, b"aa");

        let key = to_vec(&0xaabbu16).unwrap();
        assert_eq!(&key, b"aabb");

        let key = to_vec(&0xaabbccddu32).unwrap();
        assert_eq!(&key, b"aabbccdd");

        let key = to_vec(&0xaabbccdd11223344u64).unwrap();
        assert_eq!(&key, b"aabbccdd11223344");

        let key = to_vec(&0xaabbccdd11223344eeffabcd55667788u128).unwrap();
        assert_eq!(&key, b"aabbccdd11223344eeffabcd55667788");
    }

    #[test]
    fn test_signed_integer() {
        let key = to_vec(&123i8).unwrap();
        assert_eq!(&key, b"fb");

        let key = to_vec(&12345i16).unwrap();
        assert_eq!(&key, b"b039");

        let key = to_vec(&1234567890i32).unwrap();
        assert_eq!(&key, b"c99602d2");

        let key = to_vec(&1234567890123i64).unwrap();
        assert_eq!(&key, b"8000011f71fb04cb");

        let key = to_vec(&123456789012345678901234567890i128).unwrap();
        assert_eq!(&key, b"800000018ee90ff6c373e0ee4e3f0ad2");

        let mut keys = Vec::new();
        for num in i8::MIN..=i8::MAX {
            keys.push(to_vec(&num).unwrap());
        }

        assert!(is_sorted(&keys));
    }

    #[test]
    fn test_float() {
        let key = to_vec(&1234.56f32).unwrap();
        assert_eq!(&key, b"c49a51ec");

        let key = to_vec(&1234.5678f64).unwrap();
        assert_eq!(&key, b"c0934a456d5cfaad");

        let key1 = to_vec(&-123.456f32).unwrap();
        let key2 = to_vec(&0.123f32).unwrap();
        assert!(key1 < key2);
    }

    #[test]
    fn test_char() {
        let key = to_vec(&'🐺').unwrap();
        assert_eq!(&key, b"\xF0\x9F\x90\xBA");
    }

    #[test]
    fn test_string() {
        let key = to_vec(&"hello world!").unwrap();
        assert_eq!(&key, b"hello world!");
    }

    #[test]
    fn test_bytes() {
        let key = to_vec(&Bytes::new(b"\xca\xfe")).unwrap();
        assert_eq!(&key, b"cafe");

        let key = to_vec(&ByteBuf::from(b"\xca\xfe".to_vec())).unwrap();
        assert_eq!(&key, b"cafe");
    }

    #[test]
    fn test_option() {
        assert!(to_vec(&Option::<i32>::None).is_err());
    }

    #[test]
    fn test_unit() {
        let key = to_vec(&()).unwrap();
        assert_eq!(&key, b"");
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Serialize)]
        struct MyStruct;

        let key = to_vec(&MyStruct).unwrap();
        assert_eq!(&key, b"");
    }

    #[test]
    fn test_unit_variant() {
        #[derive(Serialize)]
        #[allow(dead_code)]
        enum MyEnum {
            Hello,
            World,
        }

        let key = to_vec(&MyEnum::World).unwrap();
        assert_eq!(&key, b"World");
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Serialize)]
        struct MyStruct(u16);

        let key = to_vec(&MyStruct(2)).unwrap();
        assert_eq!(&key, b"0002");
    }

    #[test]
    fn test_newtype_variant() {
        #[derive(Serialize)]
        enum MyEnum {
            Hello(u8),
        }

        assert!(to_vec(&MyEnum::Hello(1)).is_err())
    }

    #[test]
    fn test_seq() {
        let seq = vec![123, 456];

        assert!(to_vec(&seq).is_err());
    }

    #[test]
    fn test_tuple() {
        let key = to_vec(&("hello world", 2u16)).unwrap();
        assert_eq!(&key, b"hello world:0002");
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Serialize)]
        struct MyStruct(&'static str, u16);

        let key = to_vec(&MyStruct("hello world", 2u16)).unwrap();
        assert_eq!(&key, b"hello world:0002");
    }

    #[test]
    fn test_tuple_variant() {
        #[derive(Serialize)]
        enum MyEnum {
            Hello(u8, u8),
        }

        assert!(to_vec(&MyEnum::Hello(1, 2)).is_err());
    }

    #[test]
    fn test_map() {
        let mut map = HashMap::new();
        map.insert(1, 2);

        assert!(to_vec(&map).is_err());
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct MyStruct {
            s: &'static str,
            i: u8,
        }

        let key = to_vec(&MyStruct { s: "hello", i: 1 }).unwrap();
        assert_eq!(&key, b"hello:01");
    }

    #[test]
    fn test_struct_variant() {
        #[derive(Serialize)]
        enum MyEnum {
            Hello { a: u8 },
        }

        assert!(to_vec(&MyEnum::Hello { a: 1 }).is_err());
    }

    #[test]
    fn test_write() {
        let mut key = Vec::new();

        to_writer(&mut key, &"hello").unwrap();

        assert_eq!(&key, b"hello");
    }

    #[test]
    fn test_deliminator() {
        let mut key = Vec::new();
        let mut serializer = Serializer::new(&mut key).with_deliminator("/");

        assert_eq!(serializer.deliminator(), "/");

        ("hello", "world").serialize(&mut serializer).unwrap();

        serializer.into_inner();

        assert_eq!(&key, b"hello/world");
    }

    #[test]
    fn test_deliminator_and_tuple_nesting() {
        let key = to_vec(&(("hello", "world"), (1u8, 2u8), ((), ()))).unwrap();
        assert_eq!(&key, b"hello:world:01:02");
    }

    // https://stackoverflow.com/a/51272639/1524507
    fn is_sorted<T>(data: &[T]) -> bool
    where
        T: Ord,
    {
        data.windows(2).all(|w| w[0] <= w[1])
    }
}
