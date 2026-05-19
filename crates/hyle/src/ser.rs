use indexmap::IndexMap;
use serde::Serialize;
use serde::ser::Error as _;

use crate::Value;

type Result<T> = std::result::Result<T, serde::de::value::Error>;
type Error = serde::de::value::Error;

pub struct ValueSerializer;

impl serde::Serializer for ValueSerializer {
	type Ok = Value;
	type Error = Error;
	type SerializeSeq = ValueSeqSerializer;
	type SerializeTuple = ValueSeqSerializer;
	type SerializeTupleStruct = ValueSeqSerializer;
	type SerializeTupleVariant = ValueVariantSerializer;
	type SerializeMap = ValueMapSerializer;
	type SerializeStruct = ValueMapSerializer;
	type SerializeStructVariant = ValueVariantSerializer;

	fn serialize_bool(self, v: bool) -> Result<Value> {
		Ok(Value::Bool(v))
	}
	fn serialize_i8(self, v: i8) -> Result<Value> { Ok(Value::Int(v as i64)) }
	fn serialize_i16(self, v: i16) -> Result<Value> { Ok(Value::Int(v as i64)) }
	fn serialize_i32(self, v: i32) -> Result<Value> { Ok(Value::Int(v as i64)) }
	fn serialize_i64(self, v: i64) -> Result<Value> { Ok(Value::Int(v)) }
	fn serialize_u8(self, v: u8) -> Result<Value> { Ok(Value::Int(v as i64)) }
	fn serialize_u16(self, v: u16) -> Result<Value> { Ok(Value::Int(v as i64)) }
	fn serialize_u32(self, v: u32) -> Result<Value> { Ok(Value::Int(v as i64)) }
	fn serialize_u64(self, v: u64) -> Result<Value> { Ok(Value::Int(v as i64)) }

	fn serialize_f32(self, v: f32) -> Result<Value> {
		Ok(Value::Float(v as f64))
	}
	fn serialize_f64(self, v: f64) -> Result<Value> {
		Ok(Value::Float(v))
	}
	fn serialize_char(self, v: char) -> Result<Value> {
		Ok(Value::String(v.to_string()))
	}
	fn serialize_str(self, v: &str) -> Result<Value> {
		Ok(Value::String(v.to_owned()))
	}
	fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
		Ok(Value::Bytes(v.to_vec()))
	}
	fn serialize_none(self) -> Result<Value> {
		Ok(Value::Null)
	}
	fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<Value> {
		v.serialize(self)
	}
	fn serialize_unit(self) -> Result<Value> {
		Ok(Value::Null)
	}
	fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
		Ok(Value::Null)
	}
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_idx: u32,
		_variant: &'static str,
	) -> Result<Value> {
		Ok(Value::Null)
	}
	fn serialize_newtype_struct<T: ?Sized + Serialize>(
		self,
		_name: &'static str,
		v: &T,
	) -> Result<Value> {
		v.serialize(self)
	}
	fn serialize_newtype_variant<T: ?Sized + Serialize>(
		self,
		_name: &'static str,
		_idx: u32,
		_variant: &'static str,
		v: &T,
	) -> Result<Value> {
		v.serialize(self)
	}
	fn serialize_seq(self, _len: Option<usize>) -> Result<ValueSeqSerializer> {
		Ok(ValueSeqSerializer(Vec::new()))
	}
	fn serialize_tuple(self, len: usize) -> Result<ValueSeqSerializer> {
		self.serialize_seq(Some(len))
	}
	fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<ValueSeqSerializer> {
		self.serialize_seq(Some(len))
	}
	fn serialize_map(self, _len: Option<usize>) -> Result<ValueMapSerializer> {
		Ok(ValueMapSerializer {
			map: IndexMap::new(),
			key: None,
		})
	}
	fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<ValueMapSerializer> {
		self.serialize_map(None)
	}
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_idx: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<ValueVariantSerializer> {
		Ok(ValueVariantSerializer)
	}
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_idx: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<ValueVariantSerializer> {
		Ok(ValueVariantSerializer)
	}
}

pub struct ValueSeqSerializer(Vec<Value>);

impl serde::ser::SerializeSeq for ValueSeqSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
		self.0.push(v.serialize(ValueSerializer)?);
		Ok(())
	}
	fn end(self) -> Result<Value> {
		Ok(Value::Array(self.0))
	}
}

impl serde::ser::SerializeTuple for ValueSeqSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
		serde::ser::SerializeSeq::serialize_element(self, v)
	}
	fn end(self) -> Result<Value> {
		serde::ser::SerializeSeq::end(self)
	}
}

impl serde::ser::SerializeTupleStruct for ValueSeqSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
		serde::ser::SerializeSeq::serialize_element(self, v)
	}
	fn end(self) -> Result<Value> {
		serde::ser::SerializeSeq::end(self)
	}
}

pub struct ValueMapSerializer {
	map: IndexMap<String, Value>,
	key: Option<String>,
}

impl serde::ser::SerializeMap for ValueMapSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_key<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
		let key_val = v.serialize(ValueSerializer)?;
		self.key = Some(match key_val {
			Value::String(s) => s,
			other => other.to_string(),
		});
		Ok(())
	}
	fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
		let key = self.key.take().ok_or_else(|| Error::custom("missing key"))?;
		self.map.insert(key, v.serialize(ValueSerializer)?);
		Ok(())
	}
	fn end(self) -> Result<Value> {
		Ok(Value::Map(self.map))
	}
}

impl serde::ser::SerializeStruct for ValueMapSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, v: &T) -> Result<()> {
		serde::ser::SerializeMap::serialize_key(self, key)?;
		serde::ser::SerializeMap::serialize_value(self, v)
	}
	fn end(self) -> Result<Value> {
		serde::ser::SerializeMap::end(self)
	}
}

pub struct ValueVariantSerializer;

impl serde::ser::SerializeTupleVariant for ValueVariantSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_field<T: ?Sized + Serialize>(&mut self, _v: &T) -> Result<()> {
		Ok(())
	}
	fn end(self) -> Result<Value> {
		Ok(Value::Null)
	}
}

impl serde::ser::SerializeStructVariant for ValueVariantSerializer {
	type Ok = Value;
	type Error = Error;
	fn serialize_field<T: ?Sized + Serialize>(
		&mut self,
		_key: &'static str,
		_v: &T,
	) -> Result<()> {
		Ok(())
	}
	fn end(self) -> Result<Value> {
		Ok(Value::Null)
	}
}
