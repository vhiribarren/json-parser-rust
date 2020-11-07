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
        self.parse_json_value()
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

    fn advance_and_validate(&mut self, token: Token) -> Result<(), JsonError> {
        let token_result = self
            .lexer
            .next()
            .ok_or_else(|| JsonError::Other(String::from("No data to parse")))??
            .token;
        if token_result == token {
            Ok(())
        } else {
            Err(self.build_parser_error(format!(
                "Was waiting {:?} but received {:?}",
                token, token_result
            )))
        }
    }

    fn parse_json_value(&mut self) -> Result<Json, JsonError> {
        let result = match &self.current_token_info.token {
            Token::ArrayStart => Json::Array(self.parse_array()?),
            Token::ObjectStart => Json::Object(self.parse_object()?),
            Token::ValueNull => Json::Null,
            Token::ValueNumber(n) => Json::Number(*n),
            Token::ValueBoolean(b) => Json::Boolean(*b),
            Token::ValueString(s) => Json::String(s.to_string()),
            other => return Err(self.build_parser_error(format!("The token '{:?}' is not valid here, was waiting the start of an array, object or a value", other))),
        };
        Ok(result)
    }

    fn parse_array(&mut self) -> Result<Vec<Json>, JsonError> {
        assert_eq!(self.current_token_info.token, Token::ArrayStart);
        let mut vec = Vec::new();
        self.advance()?;
        if let Token::ArrayEnd = self.current_token_info.token {
            return Ok(vec);
        }
        loop {
            let value = self.parse_json_value()?;
            vec.push(value);
            self.advance()?;
            match &self.current_token_info.token {
                Token::ArrayEnd => return Ok(vec),
                Token::SeparatorValue => {}
                other => {
                    return Err(self.build_parser_error(format!(
                        "Was waiting a ',' or ']' but received {:?}",
                        other
                    )))
                }
            }
            self.advance()?;
        }
    }

    fn parse_object(&mut self) -> Result<HashMap<String, Json>, JsonError> {
        assert_eq!(self.current_token_info.token, Token::ObjectStart);
        let mut map = HashMap::new();
        self.advance()?;
        if let Token::ObjectEnd = self.current_token_info.token {
            return Ok(map);
        }
        loop {
            let key = match &self.current_token_info.token {
                Token::ValueString(val) => val.clone(),
                other => {
                    return Err(self.build_parser_error(format!(
                        "Was waiting a string but received {:?}",
                        other
                    )))
                }
            };
            self.advance_and_validate(Token::SeparatorName)?;
            self.advance()?;
            let value = self.parse_json_value()?;
            map.insert(key, value);
            self.advance()?;
            match &self.current_token_info.token {
                Token::ObjectEnd => return Ok(map),
                Token::SeparatorValue => {}
                other => {
                    return Err(self.build_parser_error(format!(
                        "Was waiting a ',' or '}}' but received {:?}",
                        other
                    )))
                }
            }
            self.advance()?;
        }
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

    #[test]
    fn simple_object() {
        let input = r#" {"one": "un", "two": 2, "three": null, "four": false} "#;
        let mut map = HashMap::new();
        map.insert("one".to_string(), Json::String("un".to_string()));
        map.insert("two".to_string(), Json::Number(2.0));
        map.insert("three".to_string(), Json::Null);
        map.insert("four".to_string(), Json::Boolean(false));
        let target = Json::Object(map);
        cmp_input_and_result(input, target);
    }

    #[test]
    fn empty_object() {
        let input = r#" {} "#;
        let map = HashMap::new();
        let target = Json::Object(map);
        cmp_input_and_result(input, target);
    }

    #[test]
    fn hierarchical_object() {
        let input = r#" {"one": "un", "two": {"three": null, "four": false}} "#;
        let mut map_inner = HashMap::new();
        map_inner.insert("three".to_string(), Json::Null);
        map_inner.insert("four".to_string(), Json::Boolean(false));
        let mut map_outer = HashMap::new();
        map_outer.insert("one".to_string(), Json::String("un".to_string()));
        map_outer.insert("two".to_string(), Json::Object(map_inner));
        let target = Json::Object(map_outer);
        cmp_input_and_result(input, target);
    }

    #[test]
    fn object_with_invalid_key_is_error() {
        let input = r#" {badkey: false} "#;
        assert!(parse_json(input).is_err());
    }

    #[test]
    fn simple_array() {
        let input = r#" [1, "deux", null, true] "#;
        let mut vec = Vec::new();
        vec.push(Json::Number(1.0));
        vec.push(Json::String("deux".to_string()));
        vec.push(Json::Null);
        vec.push(Json::Boolean(true));
        let target = Json::Array(vec);
        cmp_input_and_result(input, target);
    }

    #[test]
    fn empty_array() {
        let input = r#" [] "#;
        let vec = Vec::new();
        let target = Json::Array(vec);
        cmp_input_and_result(input, target);
    }
}
