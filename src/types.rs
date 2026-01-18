//! Delbin type definitions

/// Endianness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Endian {
    #[default]
    Little,
    Big,
}

/// Scalar type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl ScalarType {
    /// Return type size (in bytes)
    pub fn size(&self) -> usize {
        match self {
            ScalarType::U8 | ScalarType::I8 => 1,
            ScalarType::U16 | ScalarType::I16 => 2,
            ScalarType::U32 | ScalarType::I32 => 4,
            ScalarType::U64 | ScalarType::I64 => 8,
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "u8" => Some(ScalarType::U8),
            "u16" => Some(ScalarType::U16),
            "u32" => Some(ScalarType::U32),
            "u64" => Some(ScalarType::U64),
            "i8" => Some(ScalarType::I8),
            "i16" => Some(ScalarType::I16),
            "i32" => Some(ScalarType::I32),
            "i64" => Some(ScalarType::I64),
            _ => None,
        }
    }
}

/// Runtime value
#[derive(Debug, Clone)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Bytes(Vec<u8>),
    String(String),
}

impl Value {
    /// Convert to u64
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U8(v) => Some(*v as u64),
            Value::U16(v) => Some(*v as u64),
            Value::U32(v) => Some(*v as u64),
            Value::U64(v) => Some(*v),
            Value::I8(v) => Some(*v as u64),
            Value::I16(v) => Some(*v as u64),
            Value::I32(v) => Some(*v as u64),
            Value::I64(v) => Some(*v as u64),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to byte array
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }
}
