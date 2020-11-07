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

use crate::lexer::{Lexer, Token, TokenInfo};
use crate::JsonError;
use std::collections::HashMap;

// TODO Should I reimplement PartialEq to allow for float comparison?
#[derive(Debug, PartialEq)]
pub enum Json {
    Object(HashMap<String, Json>),
    Array(Vec<Json>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub fn parse_json(input: &str) -> Result<Json, JsonError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer)?;
    parser.parse()
}

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub current_token_info: TokenInfo,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self, JsonError> {
        let token_info_result = lexer
            .next()
            .ok_or_else(|| JsonError::Other(String::from("No data to parse")))?;
        let current_token_info = token_info_result?;
        Ok(Parser {
            lexer,
            current_token_info,
        })
    }

    pub fn parse(&mut self) -> Result<Json, JsonError> {
        match self.parse_json_value()? {
            None => Err(JsonError::Other(String::from("There is no data to parse"))),
            Some(json) => Ok(json),
        }
    }

    fn build_parser_error(&self, message: String) -> JsonError {
        JsonError::Parser {
            message,
            context: self.current_token_info.context.clone(),
        }
    }

    fn advance(&mut self) -> Result<(), JsonError> {
        let token_info_result = self
            .lexer
            .next()
            .ok_or_else(|| JsonError::Other(String::from("No data to parse")))?;
        Ok(self.current_token_info = token_info_result?)
    }

    fn parse_json_value(&mut self) -> Result<Option<Json>, JsonError> {
        let result = match &self.current_token_info.token {
            Token::ArrayStart => Json::Array(self.parse_array()?),
            Token::ObjectStart => Json::Object(self.parse_object()?),
            Token::ValueNull => Json::Null,
            Token::ValueNumber(n) => Json::Number(*n),
            Token::ValueBoolean(b) => Json::Boolean(*b),
            Token::ValueString(s) => Json::String(s.to_string()),
            other => return Err(self.build_parser_error(format!("The token '{:?}' is not valid here, was waiting the start of an array, object or a value", other))),
        };
        Ok(Some(result))
    }

    fn parse_array(&mut self) -> Result<Vec<Json>, JsonError> {
        unimplemented!()
    }

    fn parse_object(&mut self) -> Result<HashMap<String, Json>, JsonError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmp_input_and_result(input: &str, waited_result: Json) {
        let result = parse_json(input).unwrap();
        assert_eq!(result, waited_result);
    }

    #[test]
    fn simple_string() {
        let input = r#" "hello" "#;
        let target = Json::String(String::from("hello"));
        cmp_input_and_result(input, target);
    }

    #[test]
    fn simple_number() {
        let input = r#" 1e3 "#;
        let target = Json::Number(1e3);
        cmp_input_and_result(input, target);
    }

    #[test]
    fn simple_null() {
        let input = r#" null "#;
        let target = Json::Null;
        cmp_input_and_result(input, target);
    }
}
