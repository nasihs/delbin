//! Delbin evaluator

use std::collections::HashMap;

use crate::ast::*;
use crate::builtin;
use crate::error::{DelbinError, DelbinWarning, ErrorCode, Result};
use crate::types::{Endian, ScalarType, Value};

/// Pending field (for two-phase evaluation)
#[derive(Debug)]
#[allow(dead_code)]
struct PendingField {
    name: String,
    offset: usize,
    size: usize,
    expr: Expr,
    ty: Type,
}

/// Evaluation context
pub struct Evaluator {
    /// Environment variables
    env: HashMap<String, Value>,
    /// External section data
    sections: HashMap<String, Vec<u8>>,
    /// Endianness
    endian: Endian,
    /// Current offset
    current_offset: usize,
    /// Field offset mapping
    field_offsets: HashMap<String, usize>,
    /// Current field being processed
    current_field: Option<String>,
    /// Output buffer
    output: Vec<u8>,
    /// Pending fields (self-referencing)
    pending: Vec<PendingField>,
    /// Warning list
    warnings: Vec<DelbinWarning>,
    /// Struct total size (for @sizeof(@self))
    struct_size: Option<usize>,
}

impl Evaluator {
    pub fn new(
        env: HashMap<String, Value>,
        sections: HashMap<String, Vec<u8>>,
    ) -> Self {
        Self {
            env,
            sections,
            endian: Endian::Little,
            current_offset: 0,
            field_offsets: HashMap::new(),
            current_field: None,
            output: Vec::new(),
            pending: Vec::new(),
            warnings: Vec::new(),
            struct_size: None,
        }
    }

    /// Execute evaluation
    pub fn eval(&mut self, file: &File) -> Result<Vec<u8>> {
        self.endian = file.endian;

        // First pass: calculate struct size
        self.struct_size = Some(self.calculate_struct_size(&file.struct_def)?);

        // Second pass: generate data
        self.eval_struct(&file.struct_def)?;

        // Process pending fields
        self.process_pending()?;

        Ok(std::mem::take(&mut self.output))
    }

    /// Get warnings
    pub fn warnings(&self) -> &[DelbinWarning] {
        &self.warnings
    }

    /// Calculate struct size (pre-scan)
    fn calculate_struct_size(&mut self, struct_def: &StructDef) -> Result<usize> {
        let mut offset = 0;

        for field in &struct_def.fields {
            self.current_field = Some(field.name.clone());
            self.field_offsets.insert(field.name.clone(), offset);

            let size = self.calculate_field_size(&field.ty)?;
            offset += size;
        }

        self.current_field = None;
        self.current_offset = 0;
        self.field_offsets.clear();

        Ok(offset)
    }

    /// Calculate field size
    fn calculate_field_size(&mut self, ty: &Type) -> Result<usize> {
        match ty {
            Type::Scalar(scalar) => Ok(scalar.size()),
            Type::Array { elem, len } => {
                // Temporarily set current_offset for @offsetof self-reference
                self.current_offset = *self.field_offsets.get(self.current_field.as_ref().unwrap()).unwrap();
                let len_val = self.eval_expr(len)?;
                Ok(elem.size() * len_val as usize)
            }
        }
    }

    /// Evaluate struct
    fn eval_struct(&mut self, struct_def: &StructDef) -> Result<()> {
        for field in &struct_def.fields {
            self.eval_field(field)?;
        }
        Ok(())
    }

    /// Evaluate field
    fn eval_field(&mut self, field: &FieldDef) -> Result<()> {
        self.current_field = Some(field.name.clone());
        self.field_offsets.insert(field.name.clone(), self.current_offset);

        let size = self.get_field_size(&field.ty)?;

        if let Some(init) = &field.init {
            if self.is_self_referencing(init, &field.name) {
                // Self-referencing field, fill with 0 first, process later
                let zeros = vec![0u8; size];
                self.output.extend_from_slice(&zeros);
                self.pending.push(PendingField {
                    name: field.name.clone(),
                    offset: self.current_offset,
                    size,
                    expr: init.clone(),
                    ty: field.ty.clone(),
                });
            } else {
                // Normal field, evaluate directly
                let bytes = self.eval_field_value(&field.ty, init)?;
                self.output.extend_from_slice(&bytes);
            }
        } else {
            // No initialization, fill with 0
            let zeros = vec![0u8; size];
            self.output.extend_from_slice(&zeros);
        }

        self.current_offset += size;
        self.current_field = None;

        Ok(())
    }

    /// Get field size
    fn get_field_size(&mut self, ty: &Type) -> Result<usize> {
        match ty {
            Type::Scalar(scalar) => Ok(scalar.size()),
            Type::Array { elem, len } => {
                let len_val = self.eval_expr(len)?;
                Ok(elem.size() * len_val as usize)
            }
        }
    }

    /// Check if expression self-references current field
    fn is_self_referencing(&self, expr: &Expr, field_name: &str) -> bool {
        match expr {
            Expr::Call { name, args } => {
                if name == "crc32" || name == "sha256" {
                    for arg in args {
                        if let Expr::Range { end: Some(end), .. } = arg {
                            if end == field_name {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Evaluate field value
    fn eval_field_value(&mut self, ty: &Type, init: &Expr) -> Result<Vec<u8>> {
        match ty {
            Type::Scalar(scalar) => {
                let value = self.eval_expr(init)?;
                Ok(self.scalar_to_bytes(*scalar, value))
            }
            Type::Array { elem, len } => {
                let len_val = self.eval_expr(len)? as usize;

                match init {
                    Expr::Call { name, args } if name == "bytes" => {
                        // @bytes("string")
                        if args.len() != 1 {
                            return Err(DelbinError::new(
                                ErrorCode::E04004,
                                "@bytes() requires exactly 1 argument",
                            ));
                        }
                        let s = self.eval_string(&args[0])?;
                        let (bytes, warning) = builtin::bytes(&s, len_val * elem.size());
                        if let Some(w) = warning {
                            self.warnings.push(w);
                        }
                        Ok(bytes)
                    }
                    Expr::Call { name, args } if name == "sha256" => {
                        // @sha256(section)
                        let data = self.collect_range_data(args)?;
                        let hash = builtin::sha256(&data);
                        Ok(hash.to_vec())
                    }
                    _ => {
                        // Default zero fill
                        Ok(vec![0u8; len_val * elem.size()])
                    }
                }
            }
        }
    }

    /// Evaluate expression, returns u64
    fn eval_expr(&mut self, expr: &Expr) -> Result<u64> {
        match expr {
            Expr::Number(n) => Ok(*n),

            Expr::String(_) => Err(DelbinError::new(
                ErrorCode::E03001,
                "Cannot use string as numeric value",
            )),

            Expr::EnvVar(name) => {
                let value = self.env.get(name).ok_or_else(|| {
                    DelbinError::new(ErrorCode::E02001, format!("Undefined variable: {}", name))
                })?;
                value.as_u64().ok_or_else(|| {
                    DelbinError::new(
                        ErrorCode::E03001,
                        format!("Variable '{}' is not a number", name),
                    )
                })
            }

            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                match op {
                    BinOp::Or => Ok(l | r),
                    BinOp::And => Ok(l & r),
                    BinOp::Shl => Ok(l << r),
                    BinOp::Shr => Ok(l >> r),
                    BinOp::Add => Ok(l.wrapping_add(r)),
                    BinOp::Sub => Ok(l.wrapping_sub(r)),
                }
            }

            Expr::UnaryOp { op, operand } => {
                let v = self.eval_expr(operand)?;
                match op {
                    UnaryOp::Not => Ok(!v),
                    UnaryOp::Neg => Ok((!v).wrapping_add(1)), // Two's complement
                }
            }

            Expr::Call { name, args } => self.eval_builtin_call(name, args),

            Expr::SectionRef(name) => {
                // Return section size
                let section = self.sections.get(name).ok_or_else(|| {
                    DelbinError::new(ErrorCode::E02003, format!("Undefined section: {}", name))
                })?;
                Ok(section.len() as u64)
            }

            Expr::SelfRef => {
                // @self returns current struct size
                Ok(self.struct_size.unwrap_or(0) as u64)
            }

            Expr::Range { .. } => Err(DelbinError::new(
                ErrorCode::E03001,
                "Range expression cannot be used as numeric value",
            )),
        }
    }

    /// Evaluate string expression
    fn eval_string(&mut self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::String(s) => Ok(s.clone()),
            Expr::EnvVar(name) => {
                let value = self.env.get(name).ok_or_else(|| {
                    DelbinError::new(ErrorCode::E02001, format!("Undefined variable: {}", name))
                })?;
                value.as_string().map(|s| s.to_string()).ok_or_else(|| {
                    DelbinError::new(
                        ErrorCode::E03001,
                        format!("Variable '{}' is not a string", name),
                    )
                })
            }
            _ => Err(DelbinError::new(
                ErrorCode::E03001,
                "Expected string expression",
            )),
        }
    }

    /// Evaluate built-in function call
    fn eval_builtin_call(&mut self, name: &str, args: &[Expr]) -> Result<u64> {
        match name {
            "sizeof" => {
                if args.len() != 1 {
                    return Err(DelbinError::new(
                        ErrorCode::E04004,
                        "@sizeof() requires exactly 1 argument",
                    ));
                }
                match &args[0] {
                    Expr::SelfRef => Ok(self.struct_size.unwrap_or(0) as u64),
                    Expr::SectionRef(section) | Expr::Call { name: section, .. }
                        if self.sections.contains_key(section) =>
                    {
                        Ok(self.sections[section].len() as u64)
                    }
                    // Handle simple identifier as section name
                    other => {
                        if let Expr::EnvVar(section) = other {
                            if let Some(data) = self.sections.get(section) {
                                return Ok(data.len() as u64);
                            }
                        }
                        // Try to evaluate as expression (may be section reference)
                        self.eval_expr(other)
                    }
                }
            }

            "offsetof" => {
                if args.len() != 1 {
                    return Err(DelbinError::new(
                        ErrorCode::E04004,
                        "@offsetof() requires exactly 1 argument",
                    ));
                }
                // Extract field name from argument
                let field_name = self.extract_field_name(&args[0])?;

                // Self-reference check
                if let Some(ref current) = self.current_field {
                    if &field_name == current {
                        return Ok(self.current_offset as u64);
                    }
                }

                // Find known field offset
                self.field_offsets
                    .get(&field_name)
                    .map(|&o| o as u64)
                    .ok_or_else(|| {
                        DelbinError::new(
                            ErrorCode::E02002,
                            format!("Undefined field: {}", field_name),
                        )
                    })
            }

            "crc32" => {
                let data = self.collect_range_data(args)?;
                Ok(builtin::crc32(&data) as u64)
            }

            "sha256" => {
                // sha256 returns byte array, not a number
                Err(DelbinError::new(
                    ErrorCode::E03001,
                    "@sha256() returns bytes, not a number",
                ))
            }

            "bytes" => {
                // bytes returns byte array, not a number
                Err(DelbinError::new(
                    ErrorCode::E03001,
                    "@bytes() returns bytes, not a number",
                ))
            }

            _ => Err(DelbinError::new(
                ErrorCode::E02004,
                format!("Unknown function: @{}", name),
            )),
        }
    }

    /// Extract field name from expression
    fn extract_field_name(&self, expr: &Expr) -> Result<String> {
        match expr {
            // When parsing directly, offsetof arguments may be parsed as different forms
            // Try to extract from various forms
            Expr::EnvVar(name) => Ok(name.clone()),
            Expr::SectionRef(name) => Ok(name.clone()),
            Expr::Call { name, .. } => Ok(name.clone()),
            _ => Err(DelbinError::new(
                ErrorCode::E04003,
                "Invalid argument for @offsetof()",
            )),
        }
    }

    /// Collect range data for CRC/Hash calculation
    fn collect_range_data(&self, args: &[Expr]) -> Result<Vec<u8>> {
        if args.is_empty() {
            return Err(DelbinError::new(
                ErrorCode::E04004,
                "Function requires at least 1 argument",
            ));
        }

        let mut data = Vec::new();

        for arg in args {
            match arg {
                Expr::Range { start, end, .. } => {
                    let start_offset = match start {
                        Some(expr) => self.eval_expr_const(expr)? as usize,
                        None => 0,
                    };

                    let end_offset = match end {
                        Some(field_name) => {
                            *self.field_offsets.get(field_name).ok_or_else(|| {
                                DelbinError::new(
                                    ErrorCode::E02002,
                                    format!("Undefined field: {}", field_name),
                                )
                            })?
                        }
                        None => self.output.len(),
                    };

                    if start_offset <= end_offset && end_offset <= self.output.len() {
                        data.extend_from_slice(&self.output[start_offset..end_offset]);
                    } else {
                        return Err(DelbinError::new(
                            ErrorCode::E04002,
                            format!("Invalid range: {}..{}", start_offset, end_offset),
                        ));
                    }
                }

                Expr::SelfRef => {
                    data.extend_from_slice(&self.output);
                }

                Expr::SectionRef(name) => {
                    let section = self.sections.get(name).ok_or_else(|| {
                        DelbinError::new(ErrorCode::E02003, format!("Undefined section: {}", name))
                    })?;
                    data.extend_from_slice(section);
                }

                // Section name may be parsed as other forms
                other => {
                    if let Ok(section_name) = self.extract_field_name(other) {
                        if let Some(section) = self.sections.get(&section_name) {
                            data.extend_from_slice(section);
                            continue;
                        }
                    }
                    return Err(DelbinError::new(
                        ErrorCode::E04003,
                        "Invalid argument for checksum function",
                    ));
                }
            }
        }

        Ok(data)
    }

    /// Constant expression evaluation (does not modify state)
    fn eval_expr_const(&self, expr: &Expr) -> Result<u64> {
        match expr {
            Expr::Number(n) => Ok(*n),
            _ => Err(DelbinError::new(
                ErrorCode::E04003,
                "Expected constant expression",
            )),
        }
    }

    /// Process pending fields
    fn process_pending(&mut self) -> Result<()> {
        for pending in std::mem::take(&mut self.pending) {
            let bytes = self.eval_pending_field(&pending)?;

            // Backfill data
            if pending.offset + bytes.len() <= self.output.len() {
                self.output[pending.offset..pending.offset + bytes.len()]
                    .copy_from_slice(&bytes);
            }
        }
        Ok(())
    }

    /// Evaluate pending field
    fn eval_pending_field(&mut self, pending: &PendingField) -> Result<Vec<u8>> {
        match &pending.ty {
            Type::Scalar(scalar) => {
                let value = match &pending.expr {
                    Expr::Call { name, args } if name == "crc32" => {
                        let data = self.collect_range_data(args)?;
                        builtin::crc32(&data) as u64
                    }
                    _ => self.eval_expr(&pending.expr)?,
                };
                Ok(self.scalar_to_bytes(*scalar, value))
            }
            Type::Array { elem, len } => {
                let len_val = self.eval_expr(len)? as usize;
                match &pending.expr {
                    Expr::Call { name, args } if name == "sha256" => {
                        let data = self.collect_range_data(args)?;
                        let hash = builtin::sha256(&data);
                        Ok(hash.to_vec())
                    }
                    _ => Ok(vec![0u8; len_val * elem.size()]),
                }
            }
        }
    }

    /// Convert scalar to bytes
    fn scalar_to_bytes(&self, scalar: ScalarType, value: u64) -> Vec<u8> {
        match (scalar, self.endian) {
            (ScalarType::U8, _) | (ScalarType::I8, _) => vec![value as u8],

            (ScalarType::U16, Endian::Little) | (ScalarType::I16, Endian::Little) => {
                (value as u16).to_le_bytes().to_vec()
            }
            (ScalarType::U16, Endian::Big) | (ScalarType::I16, Endian::Big) => {
                (value as u16).to_be_bytes().to_vec()
            }

            (ScalarType::U32, Endian::Little) | (ScalarType::I32, Endian::Little) => {
                (value as u32).to_le_bytes().to_vec()
            }
            (ScalarType::U32, Endian::Big) | (ScalarType::I32, Endian::Big) => {
                (value as u32).to_be_bytes().to_vec()
            }

            (ScalarType::U64, Endian::Little) | (ScalarType::I64, Endian::Little) => {
                value.to_le_bytes().to_vec()
            }
            (ScalarType::U64, Endian::Big) | (ScalarType::I64, Endian::Big) => {
                value.to_be_bytes().to_vec()
            }
        }
    }
}