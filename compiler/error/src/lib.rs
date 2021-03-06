use std::error::Error;
use std::fmt;

use dsp_python_parser::ast;

pub mod types;
pub use crate::types::LLVMCompileErrorType;

pub mod macros;
pub use crate::macros::*;

// These errors are not compatible with the parsing errors
#[derive(Debug)]
pub struct LLVMCompileError {
    pub error: LLVMCompileErrorType,
    pub location: Option<ast::Location>,
    pub file: Option<String>,
}

impl LLVMCompileError {
    pub fn new(location: Option<ast::Location>, exception: LLVMCompileErrorType) -> Self {
        LLVMCompileError {
            error: exception,
            location,
            file: None,
        }
    }
}

impl Error for LLVMCompileError {}

impl fmt::Display for LLVMCompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_desc = match &self.error {
            LLVMCompileErrorType::NameError(target) => format!("name '{}' is not defined", target),
            LLVMCompileErrorType::SyntaxError(desc) => format!("{}", desc),
            LLVMCompileErrorType::TypeError(expected, but) => {
                format!("Expected '{}', but found '{}'", expected, but)
            }
            LLVMCompileErrorType::NotImplemented(desc) => format!("{}", desc),
        };

        let loc_string = if let Some(loc) = self.location {
            format!(
                "File '{}', line {}:{}",
                self.file.as_ref().unwrap_or(&"<Unknown>".to_string()),
                loc.row(),
                loc.column()
            )
        } else {
            format!(
                "File '{}', line <Unknown>:<Unknown>",
                self.file.as_ref().unwrap_or(&"<Unknown>".to_string())
            )
        };

        eprintln!("Traceback (most recent call last):");
        eprintln!("  {}", loc_string);
        eprintln!("{}: {}", &self.error.to_string(), error_desc);

        write!(
            f,
            "LLVMCompileError: at {} -> {}: {}",
            loc_string,
            &self.error.to_string(),
            error_desc
        )
    }
}
