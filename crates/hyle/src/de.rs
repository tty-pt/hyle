use serde::de::{self, Deserializer, Visitor, SeqAccess, IntoDeserializer};
use serde::forward_to_deserialize_any;

use crate::Value;

type DeResult<T> = Result<T, de::value::Error>;

pub struct ValueDeserializer<'de> {
	value: &'de Value,
}

impl<'de> ValueDeserializer<'de> {
	pub fn new(value: &'de Value) -> Self {
		Self { value }
	}
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
	type Error = de::value::Error;

	fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
		match self.value {
			Value::Null => visitor.visit_unit(),
			Value::Bool(b) => visitor.visit_bool(*b),
			Value::Int(n) => visitor.visit_i64(*n),
			Value::Float(f) => visitor.visit_f64(*f),
			Value::String(s) => visitor.visit_borrowed_str(s),
			Value::Bytes(b) => visitor.visit_borrowed_bytes(b),
			Value::Array(arr) => visitor.visit_seq(SeqAccessor { iter: arr.iter() }),
			Value::Map(map) => visitor.visit_map(MapAccessor { iter: map.iter(), value: None }),
		}
	}

	fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
		match self.value {
			Value::Null => visitor.visit_none(),
			_ => visitor.visit_some(self),
		}
	}

	forward_to_deserialize_any! {
		bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
		byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
		map struct enum identifier ignored_any
	}
}

impl<'de> IntoDeserializer<'de, de::value::Error> for &'de Value {
	type Deserializer = ValueDeserializer<'de>;
	fn into_deserializer(self) -> Self::Deserializer {
		ValueDeserializer::new(self)
	}
}

struct SeqAccessor<'de> {
	iter: std::slice::Iter<'de, Value>,
}

impl<'de> SeqAccess<'de> for SeqAccessor<'de> {
	type Error = de::value::Error;
	fn next_element_seed<T: de::DeserializeSeed<'de>>(
		&mut self,
		seed: T,
	) -> Result<Option<T::Value>, Self::Error> {
		self.iter
			.next()
			.map(|v| seed.deserialize(ValueDeserializer::new(v)))
			.transpose()
	}
}

struct MapAccessor<'de> {
	iter: indexmap::map::Iter<'de, String, Value>,
	value: Option<&'de Value>,
}

impl<'de> de::MapAccess<'de> for MapAccessor<'de> {
	type Error = de::value::Error;
	fn next_key_seed<K: de::DeserializeSeed<'de>>(
		&mut self,
		seed: K,
	) -> Result<Option<K::Value>, Self::Error> {
		match self.iter.next() {
			Some((k, v)) => {
				self.value = Some(v);
				seed.deserialize(de::value::StrDeserializer::<Self::Error>::new(k.as_str()))
					.map(Some)
			}
			None => {
				self.value = None;
				Ok(None)
			}
		}
	}
	fn next_value_seed<V: de::DeserializeSeed<'de>>(
		&mut self,
		seed: V,
	) -> Result<V::Value, Self::Error> {
		match self.value.take() {
			Some(v) => seed.deserialize(ValueDeserializer::new(v)),
			None => Err(de::Error::custom("value without key")),
		}
	}
}
