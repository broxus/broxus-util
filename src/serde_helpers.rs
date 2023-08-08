use std::borrow::Cow;
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Serialize};

macro_rules! declare_const_helpers {
    ($($name:ident => $ty:ty),*$(,)?) => {
        $(pub const fn $name<const N: $ty>() -> $ty {
            N
        })*
    };
}

declare_const_helpers!(
    const_bool => bool,
    const_usize => usize,
    const_i8 => i8,
    const_u8 => u8,
    const_i16 => i16,
    const_u16 => u16,
    const_i32 => i32,
    const_u32 => u32,
    const_i64 => i64,
    const_u64 => u64,
    const_i128 => i128,
    const_u128 => u128,
);

pub const fn const_duration_sec<const N: u64>() -> Duration {
    Duration::from_secs(N)
}

pub const fn const_duration_ms<const N: u64>() -> Duration {
    Duration::from_millis(N)
}

pub trait JsonNumberRepr {
    #[inline(always)]
    fn fits_into_number(&self) -> bool {
        true
    }
}
impl<T: JsonNumberRepr> JsonNumberRepr for &T {
    #[inline(always)]
    fn fits_into_number(&self) -> bool {
        <T as JsonNumberRepr>::fits_into_number(*self)
    }
}

impl JsonNumberRepr for u8 {}
impl JsonNumberRepr for i8 {}
impl JsonNumberRepr for u16 {}
impl JsonNumberRepr for i16 {}
impl JsonNumberRepr for u32 {}
impl JsonNumberRepr for i32 {}
impl JsonNumberRepr for u64 {
    #[inline(always)]
    fn fits_into_number(&self) -> bool {
        *self <= 0x1fffffffffffffu64
    }
}
impl JsonNumberRepr for i64 {
    #[inline(always)]
    fn fits_into_number(&self) -> bool {
        *self <= 0x1fffffffffffffi64
    }
}
impl JsonNumberRepr for u128 {
    #[inline(always)]
    fn fits_into_number(&self) -> bool {
        *self <= 0x1fffffffffffffu128
    }
}
impl JsonNumberRepr for i128 {
    #[inline(always)]
    fn fits_into_number(&self) -> bool {
        *self <= 0x1fffffffffffffi128
    }
}

struct StringOrNumber<T>(T);

impl<T> Serialize for StringOrNumber<T>
where
    T: JsonNumberRepr + Serialize + fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !serializer.is_human_readable() || self.0.fits_into_number() {
            self.0.serialize(serializer)
        } else {
            serializer.serialize_str(&self.0.to_string())
        }
    }
}

impl<'de, T> Deserialize<'de> for StringOrNumber<T>
where
    T: FromStr + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Value<'a, T> {
            String(#[serde(borrow)] Cow<'a, str>),
            Number(T),
        }

        if deserializer.is_human_readable() {
            match Value::deserialize(deserializer)? {
                Value::String(str) => T::from_str(str.as_ref())
                    .map(Self)
                    .map_err(|_| Error::custom("Invalid number")),
                Value::Number(value) => Ok(Self(value)),
            }
        } else {
            T::deserialize(deserializer).map(StringOrNumber)
        }
    }
}

pub mod serde_string_or_number {
    use super::*;

    pub fn serialize<S, T>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: JsonNumberRepr + Serialize + fmt::Display,
    {
        StringOrNumber(data).serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromStr + Deserialize<'de>,
    {
        StringOrNumber::<T>::deserialize(deserializer).map(|StringOrNumber(x)| x)
    }
}
pub mod serde_string_or_number_optional {
    use super::*;

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: JsonNumberRepr + Serialize + fmt::Display,
    {
        match data {
            Some(data) => StringOrNumber(data).serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromStr + Deserialize<'de>,
    {
        Option::<StringOrNumber<T>>::deserialize(deserializer).map(|x| x.map(|StringOrNumber(x)| x))
    }
}

pub mod serde_duration_sec {
    use super::*;

    pub fn serialize<S>(data: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        StringOrNumber(data.as_secs()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StringOrNumber::deserialize(deserializer).map(|StringOrNumber(x)| Duration::from_secs(x))
    }
}

pub mod serde_duration_ms {
    use super::*;

    pub fn serialize<S>(data: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        StringOrNumber(data.as_millis() as u64).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StringOrNumber::deserialize(deserializer).map(|StringOrNumber(x)| Duration::from_millis(x))
    }
}

pub mod serde_base64_array {
    use super::*;

    pub fn serialize<S>(data: &dyn AsRef<[u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_base64_bytes::serialize(data, serializer)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = serde_base64_bytes::deserialize(deserializer)?;
        data.try_into()
            .map_err(|_| Error::custom(format!("Invalid array length, expected: {N}")))
    }
}

pub mod serde_hex_array {
    use super::*;

    pub fn serialize<S>(data: &dyn AsRef<[u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_hex_bytes::serialize(data, serializer)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = serde_hex_bytes::deserialize(deserializer)?;
        data.try_into()
            .map_err(|_| Error::custom(format!("Invalid array length, expected: {N}")))
    }
}

pub mod serde_optional_hex_array {
    use super::*;

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsRef<[u8]> + Sized,
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Wrapper<'a>(#[serde(with = "serde_hex_bytes")] &'a [u8]);

        match data {
            Some(data) => serializer.serialize_some(&Wrapper(data.as_ref())),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<Option<[u8; N]>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper(#[serde(with = "serde_hex_bytes")] Vec<u8>);

        let data = Option::<Wrapper>::deserialize(deserializer)?;
        Ok(match data {
            Some(data) => Some(
                data.0
                    .try_into()
                    .map_err(|_| Error::custom(format!("Invalid array length, expected: {}", N)))?,
            ),
            None => None,
        })
    }
}

pub mod serde_string {
    use super::*;

    pub fn serialize<S>(data: &dyn fmt::Display, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        data.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        <BorrowedStr>::deserialize(deserializer)
            .and_then(|data| T::from_str(data.0.as_ref()).map_err(Error::custom))
    }
}

pub mod serde_optional_string {
    use super::*;

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: fmt::Display,
    {
        data.as_ref().map(ToString::to_string).serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        Option::<BorrowedStr>::deserialize(deserializer).and_then(|data| {
            data.map(|data| T::from_str(data.0.as_ref()).map_err(Error::custom))
                .transpose()
        })
    }
}

pub mod serde_string_array {
    use super::*;

    pub fn serialize<S, T>(data: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: fmt::Display,
    {
        data.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
            .serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    where
        T: Deserialize<'de> + FromStr,
        D: serde::Deserializer<'de>,
        <T as FromStr>::Err: fmt::Display,
    {
        let BorrowedStr(s) = <_>::deserialize(deserializer)?;
        if s.contains(',') {
            let mut v = Vec::new();
            for url in s.split(',') {
                v.push(T::from_str(url).map_err(Error::custom)?);
            }
            Ok(v)
        } else {
            Ok(vec![T::from_str(s.as_ref()).map_err(Error::custom)?])
        }
    }
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        Ok(value.to_vec())
    }
}

pub mod serde_hex_bytes {
    use std::fmt;

    use serde::de::Unexpected;

    use super::*;

    pub fn serialize<S>(data: &dyn AsRef<[u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(hex::encode(data).as_str())
        } else {
            serializer.serialize_bytes(data.as_ref())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HexVisitor;

        impl<'de> Visitor<'de> for HexVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("hex-encoded byte array")
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                hex::decode(value).map_err(|_| E::invalid_type(Unexpected::Str(value), &self))
            }

            // See the `deserializing_flattened_field` test for an example why this is needed.
            fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(HexVisitor)
        } else {
            deserializer.deserialize_bytes(BytesVisitor)
        }
    }
}

pub mod serde_optional_hex_bytes {
    use super::*;

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: AsRef<[u8]>,
    {
        #[derive(serde::Serialize)]
        #[serde(transparent)]
        struct Wrapper<'a>(#[serde(with = "serde_hex_bytes")] &'a [u8]);

        match data {
            Some(data) => serializer.serialize_some(&Wrapper(data.as_ref())),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Wrapper(#[serde(with = "serde_hex_bytes")] Vec<u8>);

        Option::<Wrapper>::deserialize(deserializer).map(|wrapper| wrapper.map(|data| data.0))
    }
}

pub mod serde_base64_bytes {
    use serde::de::Unexpected;

    use super::*;

    pub fn serialize<S>(data: &dyn AsRef<[u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(base64::encode(data).as_str())
        } else {
            serializer.serialize_bytes(data.as_ref())
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Base64Visitor;

        impl<'de> Visitor<'de> for Base64Visitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("base64-encoded byte array")
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                base64::decode(value).map_err(|_| E::invalid_type(Unexpected::Str(value), &self))
            }

            // See the `deserializing_flattened_field` test for an example why this is needed.
            fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Base64Visitor)
        } else {
            deserializer.deserialize_bytes(BytesVisitor)
        }
    }
}

pub mod serde_optional_base64_bytes {
    use super::*;

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: AsRef<[u8]>,
    {
        #[derive(serde::Serialize)]
        #[serde(transparent)]
        struct Wrapper<'a>(#[serde(with = "serde_base64_bytes")] &'a [u8]);

        match data {
            Some(data) => serializer.serialize_some(&Wrapper(data.as_ref())),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Wrapper(#[serde(with = "serde_base64_bytes")] Vec<u8>);

        Option::<Wrapper>::deserialize(deserializer).map(|wrapper| wrapper.map(|data| data.0))
    }
}

pub mod serde_iter {
    pub fn serialize<S, T, V>(iter: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: IntoIterator<Item = V> + Clone,
        V: serde::Serialize,
    {
        use serde::ser::SerializeSeq;

        let iter = iter.clone().into_iter();
        let mut seq = serializer.serialize_seq(Some(iter.size_hint().0))?;
        for value in iter {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

#[derive(Deserialize)]
struct BorrowedStr<'a>(#[serde(borrow)] Cow<'a, str>);

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_string_or_number() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct Test {
            #[serde(with = "serde_string_or_number")]
            value: u64,
        }

        let test = Test { value: 123123 };
        let data = serde_json::to_string(&test).unwrap();
        assert_eq!(data, r#"{"value":123123}"#);
        assert_eq!(serde_json::from_str::<Test>(&data).unwrap(), test);

        let data = r#"{"value":"123123"}"#;
        assert_eq!(serde_json::from_str::<Test>(data).unwrap(), test);

        let test = Test {
            value: 0xffffffffffffffff,
        };
        let data = serde_json::to_string(&test).unwrap();
        assert_eq!(data, r#"{"value":"18446744073709551615"}"#);
    }

    #[test]
    fn test_changed_string() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Test {
            #[serde(with = "serde_string_array")]
            value: Vec<String>,
        }

        let test: Test = serde_json::from_str("{\"value\":\"\\\"\"}").unwrap();
        println!("{test:?}");
    }

    #[test]
    fn test_hex() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct Test {
            #[serde(with = "serde_hex_array")]
            key: [u8; 32],
        }
        let test = Test { key: [1; 32] };
        let data = serde_json::to_string(&test).unwrap();
        assert_eq!(
            data,
            r#"{"key":"0101010101010101010101010101010101010101010101010101010101010101"}"#
        );
        assert_eq!(serde_json::from_str::<Test>(&data).unwrap(), test);
        let data = bincode::serialize(&test).unwrap();
        assert!(data.len() < 64);
        assert_eq!(bincode::deserialize::<Test>(&data).unwrap(), test);
    }

    #[test]
    fn test_optional() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct Test {
            #[serde(with = "serde_optional_base64_bytes")]
            key: Option<Vec<u8>>,
        }

        let data = Test {
            key: Some(vec![1; 32]),
        };
        let res = serde_json::to_string(&data).unwrap();
        assert_eq!(
            r#"{"key":"AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE="}"#,
            res
        );
        assert_eq!(data, serde_json::from_str(&res).unwrap());

        let data = Test { key: None };
        let res = serde_json::to_string(&data).unwrap();
        assert_eq!(r#"{"key":null}"#, res);
        assert_eq!(data, serde_json::from_str(&res).unwrap())
    }

    #[test]
    fn test_optional_hex_array() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            #[serde(with = "serde_optional_hex_array")]
            field: Option<[u8; 32]>,
        }

        let target: [u8; 32] =
            hex::decode("0101010101010101010101010101010101010101010101010101010101010101")
                .unwrap()
                .try_into()
                .unwrap();

        let serialized = serde_json::to_string(&Test {
            field: Some(target),
        })
        .unwrap();
        let deserialized: Test = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.field, Some(target));

        let serialized = serde_json::to_string(&Test { field: None }).unwrap();
        let deserialized: Test = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.field, None);
    }
}
