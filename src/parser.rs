//! Delbin parser

use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;
use crate::error::{DelbinError, ErrorCode, Result};
use crate::types::{Endian, ScalarType};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct DelBinParser;

/// Parse DSL text
pub fn parse(input: &str) -> Result<File> {
    let pairs = DelBinParser::parse(Rule::file, input).map_err(|e| {
        DelbinError::new(ErrorCode::E01003, format!("Parse error: {}", e))
    })?;

    let mut endian = Endian::Little;
    let mut struct_def = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::file => {
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::directive => {
                            endian = parse_directive(inner)?;
                        }
                        Rule::struct_def => {
                            struct_def = Some(parse_struct_def(inner)?);
                        }
                        Rule::EOI => {}
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(File {
        endian,
        struct_def: struct_def.ok_or_else(|| {
            DelbinError::new(ErrorCode::E01003, "No struct definition found")
        })?,
    })
}

fn parse_directive(pair: pest::iterators::Pair<Rule>) -> Result<Endian> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::directive_value {
            return match inner.as_str() {
                "little" => Ok(Endian::Little),
                "big" => Ok(Endian::Big),
                _ => Err(DelbinError::new(
                    ErrorCode::E01003,
                    format!("Invalid endian value: {}", inner.as_str()),
                )),
            };
        }
    }
    Ok(Endian::Little)
}

fn parse_struct_def(pair: pest::iterators::Pair<Rule>) -> Result<StructDef> {
    let mut name = String::new();
    let mut packed = false;
    let mut align = None;
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                name = inner.as_str().to_string();
            }
            Rule::struct_attr => {
                let attr_str = inner.as_str();
                if attr_str.contains("packed") {
                    packed = true;
                } else if attr_str.contains("align") {
                    // Parse @align(n)
                    for attr_inner in inner.into_inner() {
                        if attr_inner.as_rule() == Rule::align_attr {
                            for num in attr_inner.into_inner() {
                                if num.as_rule() == Rule::number {
                                    align = Some(num.as_str().parse().unwrap_or(1));
                                }
                            }
                        }
                    }
                }
            }
            Rule::field_def => {
                fields.push(parse_field_def(inner)?);
            }
            _ => {}
        }
    }

    Ok(StructDef {
        name,
        packed,
        align,
        fields,
    })
}

fn parse_field_def(pair: pest::iterators::Pair<Rule>) -> Result<FieldDef> {
    let mut name = String::new();
    let mut ty = None;
    let mut init = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::ident => {
                if name.is_empty() {
                    name = inner.as_str().to_string();
                }
            }
            Rule::type_spec => {
                ty = Some(parse_type_spec(inner)?);
            }
            Rule::init_expr => {
                // Parse expr inside init_expr
                for expr_inner in inner.into_inner() {
                    if expr_inner.as_rule() == Rule::expr {
                        init = Some(parse_expr(expr_inner)?);
                    }
                }
            }
            Rule::expr => {
                init = Some(parse_expr(inner)?);
            }
            _ => {}
        }
    }

    Ok(FieldDef {
        name,
        ty: ty.ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Missing type"))?,
        init,
    })
}

fn parse_type_spec(pair: pest::iterators::Pair<Rule>) -> Result<Type> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::scalar_type => {
                let scalar = ScalarType::from_str(inner.as_str()).ok_or_else(|| {
                    DelbinError::new(ErrorCode::E01003, format!("Unknown type: {}", inner.as_str()))
                })?;
                return Ok(Type::Scalar(scalar));
            }
            Rule::array_type => {
                return parse_array_type(inner);
            }
            _ => {}
        }
    }
    Err(DelbinError::new(ErrorCode::E01003, "Invalid type"))
}

fn parse_array_type(pair: pest::iterators::Pair<Rule>) -> Result<Type> {
    let mut elem = None;
    let mut len = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::scalar_type => {
                elem = ScalarType::from_str(inner.as_str());
            }
            Rule::expr => {
                len = Some(parse_expr(inner)?);
            }
            _ => {}
        }
    }

    Ok(Type::Array {
        elem: elem.ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Missing array element type"))?,
        len: Box::new(len.ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Missing array length"))?),
    })
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    // Handle the case where we might receive an expr node or directly an or_expr node
    let actual_pair = if pair.as_rule() == Rule::expr {
        // Unwrap expr to get or_expr
        pair.into_inner().next().ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Empty expr"))?
    } else {
        pair
    };
    parse_or_expr(actual_pair)
}

fn parse_or_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    // Unwrap if necessary
    let actual_pair = if pair.as_rule() != Rule::or_expr {
        pair.into_inner().next().ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Empty or_expr"))?
    } else {
        pair
    };
    
    let mut inner_pairs: Vec<_> = actual_pair.into_inner().collect();

    if inner_pairs.is_empty() {
        return Err(DelbinError::new(ErrorCode::E01003, "Empty expression"));
    }

    let mut left = parse_and_expr(inner_pairs.remove(0))?;

    while !inner_pairs.is_empty() {
        let right = parse_and_expr(inner_pairs.remove(0))?;
        left = Expr::BinaryOp {
            op: BinOp::Or,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_and_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    // Unwrap if necessary
    let actual_pair = if pair.as_rule() != Rule::and_expr {
        pair.into_inner().next().ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Empty and_expr"))?
    } else {
        pair
    };
    
    let mut inner_pairs: Vec<_> = actual_pair.into_inner().collect();

    if inner_pairs.is_empty() {
        return Err(DelbinError::new(ErrorCode::E01003, "Empty expression"));
    }

    let mut left = parse_shift_expr(inner_pairs.remove(0))?;

    while !inner_pairs.is_empty() {
        let right = parse_shift_expr(inner_pairs.remove(0))?;
        left = Expr::BinaryOp {
            op: BinOp::And,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_shift_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    // Unwrap if necessary
    let actual_pair = if pair.as_rule() != Rule::shift_expr {
        pair.into_inner().next().ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Empty shift_expr"))?
    } else {
        pair
    };
    
    let mut inner_pairs: Vec<_> = actual_pair.into_inner().collect();

    if inner_pairs.is_empty() {
        return Err(DelbinError::new(ErrorCode::E01003, "Empty expression"));
    }

    let mut left = parse_add_expr(inner_pairs.remove(0))?;

    while inner_pairs.len() >= 2 {
        let op_pair = inner_pairs.remove(0);
        let op = match op_pair.as_str() {
            "<<" => BinOp::Shl,
            ">>" => BinOp::Shr,
            _ => return Err(DelbinError::new(ErrorCode::E01003, "Invalid shift operator")),
        };
        let right = parse_add_expr(inner_pairs.remove(0))?;
        left = Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_add_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    // Unwrap if necessary
    let actual_pair = if pair.as_rule() != Rule::add_expr {
        pair.into_inner().next().ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Empty add_expr"))?
    } else {
        pair
    };
    
    let mut inner_pairs: Vec<_> = actual_pair.into_inner().collect();

    if inner_pairs.is_empty() {
        return Err(DelbinError::new(ErrorCode::E01003, "Empty expression"));
    }

    let mut left = parse_unary_expr(inner_pairs.remove(0))?;

    while inner_pairs.len() >= 2 {
        let op_pair = inner_pairs.remove(0);
        let op = match op_pair.as_str() {
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            _ => return Err(DelbinError::new(ErrorCode::E01003, "Invalid add operator")),
        };
        let right = parse_unary_expr(inner_pairs.remove(0))?;
        left = Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_unary_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    // Unwrap if necessary
    let actual_pair = if pair.as_rule() != Rule::unary_expr {
        pair.into_inner().next().ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Empty unary_expr"))?
    } else {
        pair
    };
    
    let mut unary_op = None;
    let mut operand = None;

    for inner in actual_pair.into_inner() {
        match inner.as_rule() {
            Rule::unary_op => {
                unary_op = Some(match inner.as_str() {
                    "~" => UnaryOp::Not,
                    "-" => UnaryOp::Neg,
                    _ => return Err(DelbinError::new(ErrorCode::E01003, "Invalid unary operator")),
                });
            }
            Rule::primary_expr => {
                operand = Some(parse_primary_expr(inner)?);
            }
            _ => {}
        }
    }

    let expr = operand.ok_or_else(|| DelbinError::new(ErrorCode::E01003, "Missing operand"))?;

    if let Some(op) = unary_op {
        Ok(Expr::UnaryOp {
            op,
            operand: Box::new(expr),
        })
    } else {
        Ok(expr)
    }
}

fn parse_primary_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::builtin_call => {
                return parse_builtin_call(inner);
            }
            Rule::env_var => {
                return parse_env_var(inner);
            }
            Rule::hex_number => {
                let s = inner.as_str();
                let value = u64::from_str_radix(&s[2..], 16).map_err(|_| {
                    DelbinError::new(ErrorCode::E01004, format!("Invalid hex number: {}", s))
                })?;
                return Ok(Expr::Number(value));
            }
            Rule::bin_number => {
                let s = inner.as_str();
                let value = u64::from_str_radix(&s[2..], 2).map_err(|_| {
                    DelbinError::new(ErrorCode::E01004, format!("Invalid binary number: {}", s))
                })?;
                return Ok(Expr::Number(value));
            }
            Rule::dec_number => {
                let value = inner.as_str().parse::<u64>().map_err(|_| {
                    DelbinError::new(ErrorCode::E01004, format!("Invalid number: {}", inner.as_str()))
                })?;
                return Ok(Expr::Number(value));
            }
            Rule::string => {
                let s = inner.as_str();
                // Remove quotes and handle escapes
                let content = &s[1..s.len() - 1];
                let unescaped = unescape_string(content)?;
                return Ok(Expr::String(unescaped));
            }
            Rule::expr => {
                return parse_expr(inner);
            }
            _ => {}
        }
    }
    Err(DelbinError::new(ErrorCode::E01003, "Invalid primary expression"))
}

fn parse_builtin_call(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut name = String::new();
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::builtin_name => {
                name = inner.as_str().to_string();
            }
            Rule::arg_list => {
                args = parse_arg_list(inner)?;
            }
            _ => {}
        }
    }

    Ok(Expr::Call { name, args })
}

fn parse_arg_list(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Expr>> {
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::arg {
            args.push(parse_arg(inner)?);
        }
    }

    Ok(args)
}

fn parse_arg(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::range_expr => {
                return parse_range_expr(inner);
            }
            Rule::section_ref => {
                // Section reference (e.g. image)
                let name = inner.as_str().to_string();
                return Ok(Expr::SectionRef(name));
            }
            Rule::expr => {
                // General expression (string, number, etc.)
                return parse_expr(inner);
            }
            _ => {}
        }
    }
    Err(DelbinError::new(ErrorCode::E01003, "Invalid argument"))
}

fn parse_range_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    let mut has_range_spec = false;
    let mut start = None;
    let mut end = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::range_spec => {
                has_range_spec = true;
                for spec_inner in inner.into_inner() {
                    match spec_inner.as_rule() {
                        Rule::range_start => {
                            for expr in spec_inner.into_inner() {
                                start = Some(Box::new(parse_expr(expr)?));
                            }
                        }
                        Rule::range_end => {
                            for ident in spec_inner.into_inner() {
                                end = Some(ident.as_str().to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if has_range_spec {
        Ok(Expr::Range {
            base: Box::new(Expr::SelfRef),
            start,
            end,
        })
    } else {
        Ok(Expr::SelfRef)
    }
}

fn parse_env_var(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::ident {
            return Ok(Expr::EnvVar(inner.as_str().to_string()));
        }
    }
    Err(DelbinError::new(ErrorCode::E01003, "Invalid environment variable"))
}

/// Handle string escape sequences
fn unescape_string(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some('x') => {
                    let mut hex = String::new();
                    if let Some(h1) = chars.next() {
                        hex.push(h1);
                    }
                    if let Some(h2) = chars.next() {
                        hex.push(h2);
                    }
                    let byte = u8::from_str_radix(&hex, 16).map_err(|_| {
                        DelbinError::new(ErrorCode::E01005, format!("Invalid hex escape: \\x{}", hex))
                    })?;
                    result.push(byte as char);
                }
                Some(c) => {
                    return Err(DelbinError::new(
                        ErrorCode::E01005,
                        format!("Invalid escape sequence: \\{}", c),
                    ));
                }
                None => {
                    return Err(DelbinError::new(ErrorCode::E01005, "Unexpected end of string"));
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input = r#"
            @endian = little;
            struct header @packed {
                magic: [u8; 4] = @bytes("fpk\0");
                version: u32 = 0x0100;
            }
        "#;

        let result = parse(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.endian, Endian::Little);
        assert_eq!(file.struct_def.name, "header");
        assert!(file.struct_def.packed);
        assert_eq!(file.struct_def.fields.len(), 2);
    }
}
