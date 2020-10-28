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

use std::iter;
use std::str;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Token {
    ArrayStart,
    ArrayEnd,
    ObjectStart,
    ObjectEnd,
    SeparatorName,
    SeparatorValue,
    ValueNull,
    ValueNumber(f64),
    ValueBoolean(bool),
    ValueString(String),
    EndOfData,
}

pub struct Context {
    line: usize,
    column: usize,
}

pub struct LexerResult {
    token: Token,
    context: Context,
}

pub struct LexerError {
    message: String,
    context: Context,
}

pub struct Lexer<'a> {
    line: usize,
    column: usize,
    data_iter: iter::Peekable<str::Chars<'a>>,
}

fn string_to_unicode_char(number: &str) -> Option<char> {
    // https://stackoverflow.com/questions/40055279/parse-a-string-containing-a-unicode-number-into-the-corresponding-unicode-charac
    u32::from_str_radix(number, 16)
        .ok()
        .and_then(std::char::from_u32)
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Lexer<'a> {
        Lexer {
            line: 0,
            column: 0,
            data_iter: data.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Result<LexerResult, LexerError> {
        let c = match self.trim_whitespace_and_peek() {
            Some(val) => val,
            None => {
                return Ok(LexerResult {
                    token: Token::EndOfData,
                    context: self.build_context(),
                })
            }
        };

        let token = match c {
            'f' => {
                self.consume_seq_and_emit(&['f', 'a', 'l', 's', 'e'], Token::ValueBoolean(false))
            }
            't' => self.consume_seq_and_emit(&['t', 'r', 'u', 'e'], Token::ValueBoolean(true)),
            'n' => self.consume_seq_and_emit(&['n', 'u', 'l', 'l'], Token::ValueNull),
            ':' => self.consume_next_and_emit(Token::SeparatorName),
            ',' => self.consume_next_and_emit(Token::SeparatorValue),
            '{' => self.consume_next_and_emit(Token::ObjectStart),
            '}' => self.consume_next_and_emit(Token::ObjectEnd),
            '[' => self.consume_next_and_emit(Token::ArrayStart),
            ']' => self.consume_next_and_emit(Token::ArrayEnd),
            '"' => self.consume_string(),
            '-' | '0'..='9' => self.consume_number(),
            c => self.parse_error(String::from(c)),
        }?;
        let context = self.build_context();
        Ok(LexerResult { context, token })
    }

    fn build_context(&self) -> Context {
        Context {
            line: self.line,
            column: self.column,
        }
    }
    fn trim_whitespace_and_peek(&mut self) -> Option<char> {
        loop {
            let &candidate = self.data_iter.peek()?;
            match candidate {
                ' ' | '\t' | '\r' => self.column += 1,
                '\n' => {
                    self.column = 0;
                    self.line += 1;
                }
                _ => return Some(candidate),
            }
            self.data_iter.next();
        }
    }

    fn consume_next_and_emit(&mut self, token: Token) -> Result<Token, LexerError> {
        self.data_iter.next();
        self.column += 1;
        Ok(token)
    }

    fn consume_seq_and_emit(
        &mut self,
        pattern: &[char],
        token: Token,
    ) -> Result<Token, LexerError> {
        for &target_char in pattern.iter() {
            let candidate_char = match self.data_iter.next() {
                Some(c) => c,
                None => return self.parse_error(String::from("")),
            };
            if candidate_char != target_char {
                return self.parse_error(String::from(""));
            }
            self.column += 1;
        }
        Ok(token)
    }

    fn consume_string(&mut self) -> Result<Token, LexerError> {
        match self.data_iter.next() {
            Some('"') => (),
            _ => return self.parse_error(String::from("")),
        }
        let mut result = String::new();
        let mut is_escaping = false;
        loop {
            let c = self.data_iter.next().unwrap();
            //.ok_or(self.parse_error(String::from("")))?;
            if is_escaping {
                let transcoded_char = match c {
                    '"' => '\u{0022}',
                    '\\' => '\u{005C}',
                    '/' => '\u{002F}',
                    'b' => '\u{0008}',
                    'f' => '\u{000C}',
                    'n' => '\u{000A}',
                    'r' => '\u{000D}',
                    't' => '\u{0009}',
                    'u' => {
                        let mut unicode_char = String::new();
                        for _ in 0..4 {
                            unicode_char.push(self.data_iter.next().unwrap());
                        }
                        string_to_unicode_char(unicode_char.as_str()).unwrap()
                    }
                    _ => return self.parse_error(String::from("")),
                };
                result.push(transcoded_char);
                is_escaping = false;
                continue;
            }

            match c {
                '"' => return Ok(Token::ValueString(result)),
                '\x20' | '\x21' | '\x23'..='\x5B' | '\x5D'..='\u{10FFFF}' => result.push(c),
                '\\' => is_escaping = true,
                _ => return self.parse_error(String::from("")),
            };
        }
    }

    fn consume_number(&mut self) -> Result<Token, LexerError> {
        enum Step {
            Minus,
            IntFirst,
            Int,
            FracOrExp,
            FracFirst,
            Frac,
            ExpSign,
            ExpFirst,
            Exp,
        }
        let mut step = Step::Minus;
        let mut number = String::new();
        'outer: loop {
            let &c = match self.data_iter.peek() {
                None => break 'outer,
                Some(val) => val,
            };
            match step {
                Step::Minus => {
                    match c {
                        '-' => {
                            number.push(c);
                            self.data_iter.next();
                        }
                        '0'..='9' => (),
                        _ => return self.parse_error(String::from("")),
                    };
                    step = Step::IntFirst;
                }
                Step::IntFirst => {
                    match c {
                        '0' => step = Step::FracOrExp,
                        '1'..='9' => step = Step::Int,
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
                Step::Int => {
                    match c {
                        '.' => step = Step::FracFirst,
                        'e' | 'E' => step = Step::ExpSign,
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
                Step::FracOrExp => {
                    match c {
                        '.' => step = Step::FracFirst,
                        'e' | 'E' => step = Step::ExpSign,
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
                Step::FracFirst => {
                    match c {
                        '0'..='9' => step = Step::Frac,
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
                Step::Frac => {
                    match c {
                        'e' | 'E' => step = Step::ExpSign,
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
                Step::ExpSign => {
                    match c {
                        '+' | '-' => {
                            number.push(c);
                            self.data_iter.next();
                        }
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    step = Step::ExpFirst
                }
                Step::ExpFirst => {
                    match c {
                        '0'..='9' => step = Step::Exp,
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
                Step::Exp => {
                    match c {
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.data_iter.next();
                }
            }
        }
        match f64::from_str(number.as_str()) {
            Ok(res) => Ok(Token::ValueNumber(res)),
            Err(_) => self.parse_error(String::from("")),
        }
    }

    fn parse_error(&self, message: String) -> Result<Token, LexerError> {
        Err(LexerError {
            context: self.build_context(),
            message,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn f64_eq(a: f64, b: f64) -> bool {
        (a - b).abs() <= std::f64::EPSILON
    }

    fn parse_and_compare_seq(input: &str, target_result: &[Token]) {
        let mut lexer = Lexer::new(input);
        for target_token in target_result.iter() {
            match lexer.next_token() {
                Ok(LexerResult { token, context: _ }) => match token {
                    Token::ValueNumber(num) => {
                        if let Token::ValueNumber(target_num) = *target_token {
                            assert!(f64_eq(num, target_num));
                        } else {
                            panic!("The token is not a ValueNumber");
                        }
                    }
                    _ => assert_eq!(token, *target_token),
                },
                Err(_) => panic!("Token is invalid, cannot be parsed."),
            };
        }
    }

    #[test]
    fn empty_string_is_eof() {
        let input_data = "";
        let target_result = [Token::EndOfData];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn whitespace_string_is_eof() {
        let input_data = " \t \n \r ";
        let target_result = [Token::EndOfData];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn eof_after_eof() {
        let input_data = "";
        let target_result = [Token::EndOfData, Token::EndOfData, Token::EndOfData];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn consume_simple_token_list_with_spaces() {
        let input_data = "\t: , [\n] }{\n \r ";
        let target_result = [
            Token::SeparatorName,
            Token::SeparatorValue,
            Token::ArrayStart,
            Token::ArrayEnd,
            Token::ObjectEnd,
            Token::ObjectStart,
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn consume_simple_token_list_without_spaces() {
        let input_data = ":,[]}{";
        let target_result = [
            Token::SeparatorName,
            Token::SeparatorValue,
            Token::ArrayStart,
            Token::ArrayEnd,
            Token::ObjectEnd,
            Token::ObjectStart,
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn consume_simple_value_list_with_spaces() {
        let input_data = "null false true";
        let target_result = [
            Token::ValueNull,
            Token::ValueBoolean(false),
            Token::ValueBoolean(true),
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn consume_simple_value_list_without_spaces() {
        let input_data = "nullfalsetrue";
        let target_result = [
            Token::ValueNull,
            Token::ValueBoolean(false),
            Token::ValueBoolean(true),
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn bad_token_is_error() {
        let input_data = " nugget ";
        let mut lexer = Lexer::new(input_data);
        assert!(matches!(
            lexer.next_token(),
            Err(LexerError { message, context })
        ));
    }

    #[test]
    fn simple_string() {
        let input_data = "  \"hello\"  \"world\"  ";
        let target_result = [
            Token::ValueString(String::from("hello")),
            Token::ValueString(String::from("world")),
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn string_with_escapes() {
        let input_data = "\"hel\\\"lo\"  \"wor\\tld\"  ";
        let target_result = [
            Token::ValueString(String::from("hel\"lo")),
            Token::ValueString(String::from("wor\tld")),
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn bad_string_escape_is_error() {
        let input_data = "\"hel\"lo\"  \"wor\\tld\"  ";
        let mut lexer = Lexer::new(&input_data);
        lexer.next_token();
        assert!(matches!(
            lexer.next_token(),
            Err(LexerError { message, context })
        ));
    }

    #[test]
    fn string_with_unicode() {
        let input_data = "\"go: 碁, cat: 🐱\"";
        let target_result = [Token::ValueString(String::from("go: 碁, cat: 🐱"))];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn string_with_escaped_basic_plan_unicode() {
        // Also test the usage of lower & upper cases for escaped unicode
        let input_data = "\"go: \\u7881\"";
        let target_result = [Token::ValueString(String::from("go: 碁")), Token::EndOfData];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn string_with_escaped_surrogate_pairs() {
        // Also test the usage of lower & upper cases for escaped unicode
        let input_data = "\"cat: \\uD83D\\udc31\"";
        let target_result = [
            Token::ValueString(String::from("cat: 🐱")),
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn number_parsing() {
        // Also test the usage of lower & upper cases for escaped unicode
        let input_data = "321 -21 54.321 -54.321 -12.34e+5 12.34e-5 -12.34e5";
        let target_result = [
            Token::ValueNumber(321.),
            Token::ValueNumber(-21.),
            Token::ValueNumber(54.321),
            Token::ValueNumber(-54.321),
            Token::ValueNumber(-12.34e+5),
            Token::ValueNumber(12.34e-5),
            Token::ValueNumber(-12.34e5),
            Token::EndOfData,
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }
}
