//! Delbin AST definitions

use crate::types::{Endian, ScalarType};

/// File (top-level)
#[derive(Debug, Clone)]
pub struct File {
    pub endian: Endian,
    pub struct_def: StructDef,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub packed: bool,
    pub align: Option<u32>,
    pub fields: Vec<FieldDef>,
}

/// Field definition
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: Type,
    pub init: Option<Expr>,
}

/// Type
#[derive(Debug, Clone)]
pub enum Type {
    Scalar(ScalarType),
    Array {
        elem: ScalarType,
        len: Box<Expr>,
    },
}

impl Type {
    /// Get element type (for arrays)
    pub fn elem_type(&self) -> ScalarType {
        match self {
            Type::Scalar(s) => *s,
            Type::Array { elem, .. } => *elem,
        }
    }
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Number literal
    Number(u64),
    /// String literal
    String(String),
    /// Environment variable reference
    EnvVar(String),
    /// Binary operation
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// Built-in function call
    Call {
        name: String,
        args: Vec<Expr>,
    },
    /// Section reference (e.g. image)
    SectionRef(String),
    /// @self reference
    SelfRef,
    /// Range expression @self[..field]
    Range {
        base: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<String>,
    },
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Or,         // |
    And,        // &
    Shl,        // <<
    Shr,        // >>
    Add,        // +
    Sub,        // -
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,        // ~
    Neg,        // -
}
