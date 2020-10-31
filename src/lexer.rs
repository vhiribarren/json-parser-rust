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
}

#[derive(Clone, Default)]
pub struct Context {
    pub line: usize,
    pub column: usize,
}

pub type LexerResult = Result<Token, String>;

pub struct LexerError {
    pub message: String,
    pub context: Context,
}

pub struct Lexer<'a> {
    char_context: Context,
    token_context: Context,
    data_iter: iter::Peekable<str::Chars<'a>>,
}

fn string_to_unicode_char(number: &str) -> Option<char> {
    // https://stackoverflow.com/questions/40055279/parse-a-string-containing-a-unicode-number-into-the-corresponding-unicode-charac
    u32::from_str_radix(number, 16)
        .ok()
        .and_then(std::char::from_u32)
}

impl std::iter::Iterator for Lexer<'_> {
    type Item = LexerResult;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.trim_whitespace_and_peek()?;
        self.set_token_context();
        let result = match c {
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
            c => Err(format!("The character '{}' is unexpected", c)),
        };
        Some(result)
    }
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Lexer<'a> {
        Lexer {
            char_context: Default::default(),
            token_context: Default::default(),
            data_iter: data.chars().peekable(),
        }
    }

    pub fn last_token_context(&self) -> Context {
        self.token_context.clone()
    }

    pub fn last_char_context(&self) -> Context {
        self.char_context.clone()
    }

    fn set_token_context(&mut self) {
        self.token_context = self.char_context.clone();
    }

    fn consume_char(&mut self) -> Option<char> {
        let next_value = self.data_iter.next();
        if let Some(c) = next_value {
            match c {
                '\n' => {
                    self.char_context.column = 0;
                    self.char_context.line += 1;
                }
                _ => self.char_context.column += 1,
            }
        }
        next_value
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.data_iter.peek()
    }

    fn trim_whitespace_and_peek(&mut self) -> Option<char> {
        loop {
            match self.peek_char()? {
                ' ' | '\t' | '\r' | '\n' => self.consume_char(),
                &candidate => return Some(candidate),
            };
        }
    }

    fn consume_next_and_emit(&mut self, token: Token) -> LexerResult {
        self.consume_char();
        Ok(token)
    }

    fn consume_seq_and_emit(
        &mut self,
        pattern: &[char],
        token: Token,
    ) -> LexerResult {
        for &target_char in pattern.iter() {
            let candidate_char = self.consume_char().ok_or_else(|| {
                format!("End of stream while waiting for '{}'", target_char)
            })?;
            if candidate_char != target_char {
                return Err(format!(
                    "Unexpected char '{}', was waiting for a '{}'",
                    candidate_char, target_char
                ));
            }
        }
        Ok(token)
    }

    fn consume_string(&mut self) -> LexerResult {
        match self.consume_char() {
            Some('"') => (),
            _ => panic!("Logic error, next char should have been a '\"'"),
        }
        let mut result = String::new();
        let mut is_escaping = false;
        loop {
            let c = self.consume_char().ok_or_else(|| {
                String::from("EOF encountered while recognizing a string")
            })?;
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
                            unicode_char.push(self.consume_char().unwrap());
                        }
                        string_to_unicode_char(unicode_char.as_str()).ok_or_else(|| {
                            format!(
                                "Could not convert {} to unicode",
                                unicode_char
                            )
                        })?
                    }
                    rest => {
                        return Err(format!("'{} is not an escapable character'", rest))
                    }
                };
                result.push(transcoded_char);
                is_escaping = false;
                continue;
            }

            match c {
                '"' => return Ok(Token::ValueString(result)),
                '\x20' | '\x21' | '\x23'..='\x5B' | '\x5D'..='\u{10FFFF}' => result.push(c),
                '\\' => is_escaping = true,
                _ => return Err(String::from("Not a valid character code")),
            };
        }
    }

    fn consume_number(&mut self) -> LexerResult {
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
            let &c = match self.peek_char() {
                None => break 'outer,
                Some(val) => val,
            };
            match step {
                Step::Minus => {
                    match c {
                        '-' => {
                            number.push(c);
                            self.consume_char();
                        }
                        '0'..='9' => (),
                        _ => panic!("Logic error, next char should have been a '-' or a number"),
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
                    self.consume_char();
                }
                Step::Int => {
                    match c {
                        '.' => step = Step::FracFirst,
                        'e' | 'E' => step = Step::ExpSign,
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.consume_char();
                }
                Step::FracOrExp => {
                    match c {
                        '.' => step = Step::FracFirst,
                        'e' | 'E' => step = Step::ExpSign,
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.consume_char();
                }
                Step::FracFirst => {
                    match c {
                        '0'..='9' => step = Step::Frac,
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.consume_char();
                }
                Step::Frac => {
                    match c {
                        'e' | 'E' => step = Step::ExpSign,
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.consume_char();
                }
                Step::ExpSign => {
                    match c {
                        '+' | '-' => {
                            number.push(c);
                            self.consume_char();
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
                    self.consume_char();
                }
                Step::Exp => {
                    match c {
                        '0'..='9' => (),
                        _ => break 'outer,
                    }
                    number.push(c);
                    self.consume_char();
                }
            }
        }
        f64::from_str(number.as_str())
            .map(Token::ValueNumber)
            .map_err(|_| format!("Could not convert '{}' to a number", number))
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
            let candidate = lexer.next().expect("No more token to be retrieved");
            match candidate {
                Ok(token) => match token {
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
        let mut lexer = Lexer::new("");
        assert!(matches!(lexer.next(), None));
    }

    #[test]
    fn whitespace_string_is_eof() {
        let mut lexer = Lexer::new( " \t \n \r ");
        assert!(matches!(lexer.next(), None));
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
            lexer.next(),
            Some(Err(_))
        ));
    }

    #[test]
    fn simple_string() {
        let input_data = "  \"hello\"  \"world\"  ";
        let target_result = [
            Token::ValueString(String::from("hello")),
            Token::ValueString(String::from("world")),
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn string_with_escapes() {
        let input_data = "\"hel\\\"lo\"  \"wor\\tld\"  ";
        let target_result = [
            Token::ValueString(String::from("hel\"lo")),
            Token::ValueString(String::from("wor\tld")),
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn bad_string_escape_is_error() {
        let input_data = "\"hel\"lo\"  \"wor\\tld\"  ";
        let mut lexer = Lexer::new(&input_data);
        lexer.next();
        assert!(matches!(
            lexer.next(),
            Some(Err(_))
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
        let target_result = [Token::ValueString(String::from("go: 碁"))];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn string_with_escaped_surrogate_pairs() {
        // Also test the usage of lower & upper cases for escaped unicode
        let input_data = "\"cat: \\uD83D\\udc31\"";
        let target_result = [
            Token::ValueString(String::from("cat: 🐱")),
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }

    #[test]
    fn number_parsing() {
        // Also test the usage of lower & upper cases for escaped unicode
        let input_data = "321 -21 0.42 54.321 -54.321 -12.34e+5 12.34e-5 -12.34e5";
        let target_result = [
            Token::ValueNumber(321.),
            Token::ValueNumber(-21.),
            Token::ValueNumber(0.42),
            Token::ValueNumber(54.321),
            Token::ValueNumber(-54.321),
            Token::ValueNumber(-12.34e+5),
            Token::ValueNumber(12.34e-5),
            Token::ValueNumber(-12.34e5),
        ];
        parse_and_compare_seq(&input_data, &target_result);
    }
}
