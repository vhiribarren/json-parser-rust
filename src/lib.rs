/*
Copyright (c) 2020 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

mod lexer;
mod parser;

pub use lexer::Lexer;
pub use parser::parse_json;
pub use parser::{Json, Parser};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Context {
    pub line: usize,
    pub column: usize,
}

impl Default for Context {
    fn default() -> Self {
        Self {line: 1, column: 1,}
    }
}

#[derive(Debug)]
pub enum JsonError {
    Lexer { context: Context, message: String },
    Parser { context: Context, message: String },
    Other(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::Lexer { context, message } => write!(
                f,
                "Lexer error, line {} column {}: {}",
                context.line, context.column, message
            ),
            JsonError::Parser { context, message } => write!(
                f,
                "Parser error, line {} column {}: {}",
                context.line, context.column, message
            ),
            JsonError::Other(message) => write!(f, "Other error: {}", message),
        }
    }
}
