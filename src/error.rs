//! Delbin error type definitions

use thiserror::Error;

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Parse errors (01)
    E01001, // UnexpectedToken
    E01002, // UnexpectedEOF
    E01003, // InvalidSyntax
    E01004, // InvalidNumber
    E01005, // InvalidString

    // Semantic errors (02)
    E02001, // UndefinedVariable
    E02002, // UndefinedField
    E02003, // UndefinedSection
    E02004, // UndefinedFunction

    // Type errors (03)
    E03001, // TypeMismatch
    E03002, // ArraySizeMismatch
    E03003, // IntegerOverflow
    E03004, // InvalidArraySize
    E03005, // StringTooLong

    // Evaluation errors (04)
    E04001, // DivisionByZero
    E04002, // InvalidRange
    E04003, // InvalidArgument
    E04004, // ArgumentCountMismatch
    E04005, // ComputationFailed
    E04006, // ShiftOverflow

    // IO errors (05)
    E05001, // FileNotFound
    E05002, // FileReadError
    E05003, // FileWriteError
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Source code location
#[derive(Debug, Clone, Default)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub context: String,
}

/// Delbin error
#[derive(Debug, Error)]
#[error("[{code}] {message}")]
pub struct DelbinError {
    pub code: ErrorCode,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub hint: Option<String>,
}

impl DelbinError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            hint: None,
        }
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Delbin warning
#[derive(Debug, Clone)]
pub struct DelbinWarning {
    pub code: WarningCode,
    pub message: String,
    pub location: Option<SourceLocation>,
}

/// Warning codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningCode {
    W03001, // StringTruncated
    W03002, // ValueTruncated
}

pub type Result<T> = std::result::Result<T, DelbinError>;
