use std::{collections::BTreeMap, fmt::Display, hash::Hash};

use possibly::possibly;
use serde::de::{Expected, Unexpected};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Object(BTreeMap<String, Value>),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Null,
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Array(arr) => arr.hash(state),
            Self::Object(obj) => obj.hash(state),
            Self::Bool(b) => b.hash(state),
            Self::Float(f) => {
                if f.is_finite() {
                    f.to_bits().hash(state);
                } else {
                    panic!("non finite floats are not supported for hashing")
                }
            }
            Self::Unsigned(i) => i.hash(state),
            Self::Signed(i) => i.hash(state),
            Self::Null => std::mem::discriminant(self).hash(state),
            Self::String(s) => s.hash(state),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Value {
        self
    }
}

impl Value {
    pub fn string<S: AsRef<str>>(str: S) -> Value {
        Self::String(str.as_ref().to_owned())
    }

    pub fn float(f: f64) -> Value {
        Self::Float(f)
    }

    pub fn object<S: AsRef<str>, V: Into<Value>, A: IntoIterator<Item = (S, V)>>(
        mapping: A,
    ) -> Value {
        Self::Object(
            mapping
                .into_iter()
                .map(|(k, v)| (k.as_ref().to_owned(), v.into()))
                .collect(),
        )
    }

    pub fn array<V: Into<Value>, A: IntoIterator<Item = V>>(elements: A) -> Value {
        Self::Array(elements.into_iter().map(V::into).collect())
    }

    pub fn signed(i: i64) -> Value {
        Self::Signed(i)
    }

    pub fn unsigned(i: u64) -> Value {
        Self::Unsigned(i)
    }

    pub fn bool(b: bool) -> Value {
        Self::Bool(b)
    }

    pub fn as_signed(&self) -> Option<i64> {
        match self {
            Self::Signed(i) => Some(*i),
            Self::Unsigned(u) => i64::try_from(*u).ok(),
            _ => None,
        }
    }

    pub fn as_unsigned(&self) -> Option<u64> {
        possibly!(self, Self::Unsigned(i) => *i)
    }

    pub fn as_float(&self) -> Option<f64> {
        possibly!(self, Self::Float(f) => *f)
    }

    pub fn as_object(&self) -> Option<BTreeMap<&str, &Value>> {
        possibly!(self, Self::Object(obj) => obj.iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect()
        )
    }

    pub fn as_mut_object(&mut self) -> Option<BTreeMap<&str, &mut Value>> {
        possibly!(self, Self::Object(obj) => obj.iter_mut()
            .map(|(key, value)| (key.as_str(), value))
            .collect()
        )
    }

    pub fn as_vec(&self) -> Option<&[Value]> {
        possibly!(self, Self::Array(vec) => vec.as_slice())
    }

    pub fn as_vec_and_then<T, C: FnMut(&Value) -> Option<T>>(&self, mapper: C) -> Option<Vec<T>> {
        self.as_vec()
            .and_then(|vec| vec.iter().map(mapper).collect())
    }

    pub fn as_object_and_then<T, C: Fn(&Value) -> Option<T>>(
        &self,
        mapper: C,
    ) -> Option<BTreeMap<String, T>> {
        match self {
            Self::Object(object) => object
                .iter()
                .map(|(name, value)| {
                    mapper(value).map(|mapped_value| (name.to_owned(), mapped_value))
                })
                .collect(),
            _ => None,
        }
    }

    pub fn as_mut_vec(&mut self) -> Option<&mut [Value]> {
        possibly!(self, Self::Array(vec) => vec.as_mut_slice())
    }

    pub fn as_str(&self) -> Option<&str> {
        possibly!(self, Self::String(s) => s)
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(bool) => Some(*bool),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get a value at the specified object path. An object path must have the following format: "name.of.my.key".
    pub fn at<S: AsRef<str>>(&self, path: S) -> Option<&Self> {
        possibly!(self, Self::Object(obj) => match path.as_ref().split_once('.') {
            Some((curr, rest)) => obj.get(curr)?.at(rest)?,
            None => obj.get(path.as_ref())?,
        })
    }

    /// Overwrite with another [`Value`] object. Uses JSON Merge patch algorithm (RFC7396)
    pub fn overwrite<V: AsRef<Value>>(&mut self, source: V) {
        match (source.as_ref(), self) {
            (Value::Object(src), Value::Object(dst)) => {
                for (key, val) in src {
                    if val.is_null() {
                        let _ = dst.remove(key);
                        continue;
                    }

                    if let Some(existing) = dst.get_mut(key) {
                        existing.overwrite(val);
                        continue;
                    }

                    let _ = dst.insert(key.clone(), val.clone());
                }
            }
            (source, target) => {
                let _ = std::mem::replace(target, source.clone());
            }
        }
    }

    pub(crate) fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    pub(crate) fn unexpected(&self) -> Unexpected {
        match self {
            Value::Null => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(*b),
            Value::Unsigned(i) => Unexpected::Unsigned(*i),
            Value::Signed(i) => Unexpected::Signed(*i),
            Value::Float(f) => Unexpected::Float(*f),
            Value::String(s) => Unexpected::Str(s),
            Value::Array(_) => Unexpected::Seq,
            Value::Object(_) => Unexpected::Map,
        }
    }
}

impl From<i8> for Value {
    fn from(i: i8) -> Self {
        Self::Signed(i as i64)
    }
}

impl From<i16> for Value {
    fn from(i: i16) -> Self {
        Self::Signed(i as i64)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::Signed(i as i64)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Signed(i)
    }
}

impl From<u8> for Value {
    fn from(i: u8) -> Self {
        Self::Unsigned(i as u64)
    }
}

impl From<u16> for Value {
    fn from(i: u16) -> Self {
        Self::Unsigned(i as u64)
    }
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Self::Unsigned(i as u64)
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        Self::Unsigned(i)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Self::Float(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<Vec<Value>> for Value {
    fn from(vec: Vec<Value>) -> Self {
        Self::Array(vec)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Null
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(bool) => f.write_str(&bool.to_string()),
            Self::Null => f.write_str("null"),
            Self::String(str) => write!(
                f,
                "\"{}\"",
                str.replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
            ),
            Self::Array(arr) => write!(
                f,
                "[{}]",
                arr.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Object(obj) => write!(
                f,
                "{{{}}}",
                obj.iter()
                    .map(|(key, val)| format!("\"{key}\": {}", val))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Float(i) => f.write_str(i.to_string().as_str()),
            Self::Unsigned(i) => f.write_str(i.to_string().as_str()),
            Self::Signed(i) => f.write_str(i.to_string().as_str()),
        }
    }
}
