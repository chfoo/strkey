//! Deserialization
use std::{collections::VecDeque, convert::TryInto, io::Read, marker::PhantomData};

use serde::{
    de::{DeserializeOwned, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
    Deserialize,
};

use crate::error::Error;

/// Deserializer for deserializing values in strkey encoding.
///
/// Example:
///
/// ```
/// use serde::de::Deserialize;
///
/// # use strkey::{Deserializer, de::SliceReader};
/// # fn main() -> Result<(), strkey::Error> {
/// let mut deserializer = Deserializer::new(SliceReader::new(b"abc:05"));
/// let output = <(&str, u8)>::deserialize(&mut deserializer)?;
/// deserializer.end()?;
///
/// assert_eq!(output.0, "abc");
/// assert_eq!(output.1, 5u8);
/// # Ok(())
/// # }
/// ```
pub struct Deserializer<'de, R: ComponentRead<'de>> {
    input: R,
    buffer: Vec<u8>,
    _de: PhantomData<&'de ()>,
}

impl<'de, R: ComponentRead<'de>> Deserializer<'de, R> {
    /// Construct a deserializer using the given component reader.
    ///
    /// See also [`Self::from_slice`] and [`Self::from_reader`].
    pub fn new(input: R) -> Self {
        Deserializer {
            input,
            buffer: Vec::new(),
            _de: PhantomData::default(),
        }
    }

    /// Returns the deliminator used to separate values.
    pub fn deliminator(&self) -> &str {
        self.input.deliminator()
    }

    /// Sets the deliminator used to separate values.
    pub fn set_deliminator(&mut self, deliminator: &'de str) {
        self.input.set_deliminator(deliminator);
    }

    /// Sets the deliminator used to separate values and returns a new serializer.
    pub fn with_deliminator(mut self, deliminator: &'de str) -> Self {
        self.set_deliminator(deliminator);
        self
    }

    /// Validates that the reader has fully processed the given input.
    pub fn end(&mut self) -> Result<(), Error> {
        if self.input.next_component()?.is_some() {
            Err(Error::Syntax)
        } else {
            Ok(())
        }
    }

    fn next_component(&mut self) -> Result<Component<'de>, Error> {
        let component = self.input.next_component()?.ok_or(Error::Syntax)?;
        Ok(component)
    }

    fn next_component_decode_hex(&mut self) -> Result<(Component<'de>, &[u8]), Error> {
        let component = self.input.next_component()?.ok_or(Error::Syntax)?;

        self.buffer.resize(component.as_str().len() / 2, 0);

        hex::decode_to_slice(component.as_str(), &mut self.buffer)
            .map_err(|error| Error::Data(format!("{}", error)))?;

        Ok((component, &self.buffer))
    }
}

impl<'de> Deserializer<'de, SliceReader<'de>> {
    /// Construct a deserializer to deserialize the given slice.
    pub fn from_slice(input: &'de [u8]) -> Self {
        Self::new(SliceReader::new(input))
    }
}

impl<'de, R: Read> Deserializer<'de, IoReader<'de, R>> {
    /// Construct a deserializer to deserialize data from the given reader.
    pub fn from_reader(input: R) -> Self {
        Self::new(IoReader::new(input))
    }
}

impl<'de, 'a, R: ComponentRead<'de>> serde::de::Deserializer<'de> for &'a mut Deserializer<'de, R> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(component) = self.input.next_component()? {
            match component.as_str() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err(Error::Data(component.to_owned())),
            }
        } else {
            Err(Error::Syntax)
        }
    }

    /// signed integer magic https://github.com/danburkert/bytekey/blob/6980b9e33281d875f03f4c9a953b93a384eac085/src/decoder.rs#L76

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 1] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_i8(i8::from_be_bytes(buffer) ^ i8::MIN)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 2] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_i16(i16::from_be_bytes(buffer) ^ i16::MIN)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 4] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_i32(i32::from_be_bytes(buffer) ^ i32::MIN)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 8] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_i64(i64::from_be_bytes(buffer) ^ i64::MIN)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 16] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_i128(i128::from_be_bytes(buffer) ^ i128::MIN)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 1] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_u8(u8::from_be_bytes(buffer))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 2] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_u16(u16::from_be_bytes(buffer))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 4] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_u32(u32::from_be_bytes(buffer))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 8] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_u64(u64::from_be_bytes(buffer))
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 16] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;

        visitor.visit_u128(u128::from_be_bytes(buffer))
    }

    // Floating point magic https://github.com/danburkert/bytekey/blob/6980b9e33281d875f03f4c9a953b93a384eac085/src/decoder.rs#L104

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 4] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;
        let val = i32::from_be_bytes(buffer);
        let t = ((val ^ i32::MIN) >> 31) | i32::MIN;

        visitor.visit_f32(f32::from_bits((val ^ t) as u32))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (component, buffer) = self.next_component_decode_hex()?;
        let buffer: [u8; 8] = buffer
            .try_into()
            .map_err(|_| Error::Data(component.to_owned()))?;
        let val = i64::from_be_bytes(buffer);
        let t = ((val ^ i64::MIN) >> 63) | i64::MIN;

        visitor.visit_f64(f64::from_bits((val ^ t) as u64))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let component = self.next_component()?;

        if component.as_str().char_indices().count() == 1 {
            if let Some(char) = component.as_str().chars().next() {
                visitor.visit_char(char)
            } else {
                Err(Error::Data(component.to_owned()))
            }
        } else {
            Err(Error::Data(component.to_owned()))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let component = self.next_component()?;

        match component {
            Component::Owned(value) => visitor.visit_string(value),
            Component::Borrowed(value) => visitor.visit_borrowed_str(value),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let component = self.next_component()?;

        match component {
            Component::Owned(value) => visitor.visit_string(value),
            Component::Borrowed(value) => visitor.visit_borrowed_str(value),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (_component, buffer) = self.next_component_decode_hex()?;

        visitor.visit_bytes(buffer)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (_component, buffer) = self.next_component_decode_hex()?;

        visitor.visit_bytes(buffer)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.input.preload_components()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.input.preload_components()?;
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_tuple<V>(mut self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(CollectionDeserializer::new(&mut self))
    }

    fn deserialize_tuple_struct<V>(
        mut self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(CollectionDeserializer::new(&mut self))
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(CollectionDeserializer::new(&mut self))
    }

    fn deserialize_enum<V>(
        mut self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(CollectionDeserializer::new(&mut self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

struct CollectionDeserializer<'a, 'de: 'a, R: ComponentRead<'de>> {
    deserializer: &'a mut Deserializer<'de, R>,
}

impl<'a, 'de, R: ComponentRead<'de>> CollectionDeserializer<'a, 'de, R> {
    fn new(deserializer: &'a mut Deserializer<'de, R>) -> Self {
        Self { deserializer }
    }
}

impl<'de, 'a, R: ComponentRead<'de>> SeqAccess<'de> for CollectionDeserializer<'a, 'de, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.deserializer).map(Some)
    }
}

impl<'de, 'a, R: ComponentRead<'de>> MapAccess<'de> for CollectionDeserializer<'a, 'de, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        Err(Error::UnsupportedType)
    }
}

impl<'de, 'a, R: ComponentRead<'de>> EnumAccess<'de> for CollectionDeserializer<'a, 'de, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.deserializer)?;

        Ok((val, self))
    }
}

impl<'de, 'a, R: ComponentRead<'de>> VariantAccess<'de> for CollectionDeserializer<'a, 'de, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }
}

/// Represents either a owned or borrowed string (values within separators).
///
/// Intended for use only within this crate.
pub enum Component<'de> {
    /// Owned string.
    Owned(String),

    /// Borrowed string.
    Borrowed(&'de str),
}

impl<'de> Component<'de> {
    /// Get string as reference.
    pub fn as_str(&self) -> &str {
        match self {
            Component::Owned(value) => &value,
            Component::Borrowed(value) => value,
        }
    }

    /// Get string as bytes reference.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Component::Owned(value) => value.as_bytes(),
            Component::Borrowed(value) => value.as_bytes(),
        }
    }

    /// Get string as a copy.
    pub fn to_owned(&self) -> String {
        match self {
            Component::Owned(value) => value.clone(),
            Component::Borrowed(value) => value.to_string(),
        }
    }
}

/// Trait that reads components (values within separators) from an input.
///
/// This trait is not intended to be implemented outside of this crate.
pub trait ComponentRead<'de> {
    /// Return the deliminator used to separate values.
    fn deliminator(&self) -> &'de str;

    /// Set the deliminator used to separate values.
    fn set_deliminator(&mut self, deliminator: &'de str);

    /// Split input into components if it hasn't been already.
    fn preload_components(&mut self) -> Result<(), Error>;

    /// Return the next value.
    fn next_component(&mut self) -> Result<Option<Component<'de>>, Error>;
}

/// Component reader for a std io reader.
pub struct IoReader<'de, R: Read> {
    input: R,
    deliminator: &'de str,
    components: Option<VecDeque<Component<'de>>>,
}

impl<'de, R: Read> IoReader<'de, R> {
    /// Construct a component reader from a reader.
    pub fn new(input: R) -> Self {
        Self {
            input,
            deliminator: ":",
            components: None,
        }
    }
}

impl<'de, R: Read> ComponentRead<'de> for IoReader<'de, R> {
    fn deliminator(&self) -> &'de str {
        &self.deliminator
    }

    fn set_deliminator(&mut self, deliminator: &'de str) {
        self.deliminator = deliminator
    }

    fn preload_components(&mut self) -> Result<(), Error> {
        if self.components.is_none() {
            let mut buf = String::new();
            self.input.read_to_string(&mut buf)?;

            let mut components = VecDeque::new();

            if !buf.is_empty() {
                for component in buf.split(self.deliminator) {
                    components.push_back(Component::Owned(component.to_string()));
                }
            }

            self.components = Some(components);
        }

        Ok(())
    }

    fn next_component(&mut self) -> Result<Option<Component<'de>>, Error> {
        self.preload_components()?;

        let components = self.components.as_mut().unwrap();

        Ok(components.pop_front())
    }
}

/// Component reader for a slice.
pub struct SliceReader<'de> {
    input: &'de [u8],
    deliminator: &'de str,
    components: Option<VecDeque<Component<'de>>>,
}

impl<'de> SliceReader<'de> {
    /// Construct a component reader for a slice.
    pub fn new(input: &'de [u8]) -> Self {
        Self {
            input,
            deliminator: ":",
            components: None,
        }
    }
}

impl<'de> ComponentRead<'de> for SliceReader<'de> {
    fn deliminator(&self) -> &'de str {
        &self.deliminator
    }

    fn set_deliminator(&mut self, deliminator: &'de str) {
        self.deliminator = deliminator
    }

    fn preload_components(&mut self) -> Result<(), Error> {
        if self.components.is_none() {
            let decoded_str = std::str::from_utf8(self.input)?;

            let mut components = VecDeque::new();

            if !decoded_str.is_empty() {
                for component in decoded_str.split(self.deliminator) {
                    components.push_back(Component::Borrowed(component));
                }
            }

            self.components = Some(components);
        }

        Ok(())
    }

    fn next_component(&mut self) -> Result<Option<Component<'de>>, Error> {
        self.preload_components()?;

        let components = self.components.as_mut().unwrap();

        Ok(components.pop_front())
    }
}

/// Deserialize the value from a byte array slice.
pub fn from_slice<'a, T>(value: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(value);
    let output = serde::de::Deserialize::deserialize(&mut deserializer)?;
    deserializer.end()?;

    Ok(output)
}

/// Deserialize strkey encoding to produce the requested value from the given reader.
pub fn from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut deserializer = Deserializer::from_reader(reader);
    let output = serde::de::Deserialize::deserialize(&mut deserializer)?;
    deserializer.end()?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_bytes::{ByteBuf, Bytes};

    use super::*;

    #[test]
    fn test_bool() {
        let value = from_slice::<bool>(b"true").unwrap();
        assert!(value);

        let value = from_slice::<bool>(b"false").unwrap();
        assert!(!value);

        assert!(from_slice::<bool>(b"h").is_err());
    }

    #[test]
    fn test_unsigned_integer() {
        let value = from_slice::<u8>(b"aa").unwrap();
        assert_eq!(value, 0xaau8);

        let value = from_slice::<u16>(b"aabb").unwrap();
        assert_eq!(value, 0xaabbu16);

        let value = from_slice::<u32>(b"aabbccdd").unwrap();
        assert_eq!(value, 0xaabbccddu32);

        let value = from_slice::<u64>(b"aabbccdd11223344").unwrap();
        assert_eq!(value, 0xaabbccdd11223344u64);

        let value = from_slice::<u128>(b"aabbccdd11223344eeffabcd55667788").unwrap();
        assert_eq!(value, 0xaabbccdd11223344eeffabcd55667788u128);

        assert!(from_slice::<u8>(b"h").is_err());
        assert!(from_slice::<u16>(b"hh").is_err());
        assert!(from_slice::<u32>(b"hhh").is_err());
        assert!(from_slice::<u64>(b"hhhh").is_err());
        assert!(from_slice::<u128>(b"hhhhh").is_err());
    }

    #[test]
    fn test_signed_integer() {
        let value = from_slice::<i8>(b"fb").unwrap();
        assert_eq!(value, 123i8);

        let value = from_slice::<i16>(b"b039").unwrap();
        assert_eq!(value, 12345i16);

        let value = from_slice::<i32>(b"c99602d2").unwrap();
        assert_eq!(value, 1234567890i32);

        let value = from_slice::<i64>(b"8000011f71fb04cb").unwrap();
        assert_eq!(value, 1234567890123i64);

        let value = from_slice::<i128>(b"800000018ee90ff6c373e0ee4e3f0ad2").unwrap();
        assert_eq!(value, 123456789012345678901234567890i128);

        assert!(from_slice::<i8>(b"h").is_err());
        assert!(from_slice::<i16>(b"hh").is_err());
        assert!(from_slice::<i32>(b"hhh").is_err());
        assert!(from_slice::<i64>(b"hhhh").is_err());
        assert!(from_slice::<i128>(b"hhhhh").is_err());
    }

    #[test]
    fn test_float() {
        let value = from_slice::<f32>(b"c49a51ec").unwrap();
        assert!((value - 1234.56f32).abs() < f32::EPSILON);

        let value = from_slice::<f64>(b"c0934a456d5cfaad").unwrap();
        assert!((value - 1234.5678f64).abs() < f64::EPSILON);

        assert!(from_slice::<f32>(b"h").is_err());
        assert!(from_slice::<f64>(b"hh").is_err());
    }

    #[test]
    fn test_char() {
        let value = from_slice::<char>(b"\xF0\x9F\x90\xBA").unwrap();
        assert_eq!(value, '🐺');

        assert!(from_slice::<char>(b"\xfe").is_err());
        assert!(from_slice::<char>(b"abc").is_err());
    }

    #[test]
    fn test_string() {
        let value = from_slice::<String>(b"hello world!").unwrap();
        assert_eq!(&value, "hello world!");

        assert!(from_slice::<String>(b"\xfe").is_err());
    }

    #[test]
    fn test_bytes() {
        //// Can't borrow from decoded hex!
        // let value = from_slice::<&Bytes>(b"cafe").unwrap();
        // assert_eq!(value, b"\xca\xfe");
        assert!(from_slice::<&Bytes>(b"cafe").is_err());

        let value = from_slice::<ByteBuf>(b"cafe").unwrap();
        assert_eq!(value, b"\xca\xfe");

        assert!(from_slice::<&Bytes>(b"h").is_err());
        assert!(from_slice::<ByteBuf>(b"hh").is_err());
    }

    #[test]
    fn test_option() {
        assert!(from_slice::<Option<i32>>(b"h").is_err());
    }

    #[test]
    fn test_unit() {
        from_slice::<()>(b"").unwrap();
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Deserialize)]
        struct MyStruct;

        from_slice::<MyStruct>(b"").unwrap();
    }

    #[test]
    fn test_unit_variant() {
        #[derive(PartialEq, Eq, Debug, Deserialize)]
        #[allow(dead_code)]
        enum MyEnum {
            Hello,
            World,
        }

        let value = from_slice::<MyEnum>(b"World").unwrap();
        assert_eq!(value, MyEnum::World);
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Deserialize)]
        struct MyStruct(u16);

        let value = from_slice::<MyStruct>(b"0002").unwrap();
        assert_eq!(value.0, 2);
    }

    #[test]
    fn test_newtype_variant() {
        #[derive(Deserialize)]
        enum MyEnum {
            Hello(u8),
        }

        assert!(from_slice::<MyEnum>(b"h").is_err());
    }

    #[test]
    fn test_seq() {
        assert!(from_slice::<Vec<i32>>(b"h").is_err());
    }

    #[test]
    fn test_tuple() {
        let value = from_slice::<(String, u16)>(b"hello world:0002").unwrap();

        assert_eq!(&value.0, "hello world");
        assert_eq!(value.1, 2);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Deserialize)]
        struct MyStruct(&'static str, u16);

        let value = from_slice::<MyStruct>(b"hello world:0002").unwrap();

        assert_eq!(value.0, "hello world");
        assert_eq!(value.1, 2);
    }

    #[test]
    fn test_tuple_variant() {
        #[derive(Deserialize)]
        enum MyEnum {
            Hello(u8, u8),
        }

        assert!(from_slice::<MyEnum>(b"h").is_err());
    }

    #[test]
    fn test_map() {
        assert!(from_slice::<HashMap<i32, i32>>(b"h").is_err());
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize)]
        struct MyStruct {
            s: &'static str,
            i: u8,
        }

        let value = from_slice::<MyStruct>(b"hello:01").unwrap();

        assert_eq!(value.s, "hello");
        assert_eq!(value.i, 1);
    }

    #[test]
    fn test_struct_variant() {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        enum MyEnum {
            Hello { a: u8 },
        }

        assert!(from_slice::<MyEnum>(b"h").is_err());
    }

    #[test]
    fn test_write() {
        let encoded = b"hello".to_vec();

        let value = from_reader::<_, String>(encoded.as_slice()).unwrap();

        assert_eq!(&value, "hello");
    }

    #[test]
    fn test_deliminator() {
        let mut deserializer = Deserializer::from_slice(b"hello/world").with_deliminator("/");

        let value = <(&str, &str)>::deserialize(&mut deserializer).unwrap();

        assert_eq!(value.0, "hello");
        assert_eq!(value.1, "world");
    }

    #[test]
    fn test_deliminator_and_tuple_nesting() {
        let value = from_slice::<((&str, &str), (u8, u8), ((), ()))>(b"hello:world:01:02").unwrap();

        assert_eq!(value, (("hello", "world"), (1u8, 2u8), ((), ())));
    }
}
