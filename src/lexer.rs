//! Lexer/Tokenizer for MCSL

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Func,  // func (for defining functions)
    If,    // if (for conditionals)
    
    // Special prefixes
    At,      // @
    Percent, // %
    Hash,    // #
    Dollar,  // $
    
    // Symbols
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Colon,    // :
    Comma,    // ,
    Equals,   // =

    // Operators
    EqEq,     // ==
    NotEq,    // !=
    Minus,    // -
    DotDot,   // ..
    Dot,      // .
    Star,     // * (wildcard)
    
    // Literals
    Identifier(String),
    String(String),
    Number(f64),
    Bool(bool),
    
    // Coordinate symbols
    Tilde,      // ~ (relative coordinate)
    Caret,      // ^ (local coordinate)
    
    // Special constructs
    EntitySelector(String), // @a, @p, @e, @s, @r
    RelativeCoord,          // @~
    LocalCoord,             // %^
    FunctionTag(String),    // $load, $tick
    
    EOF,
}

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected character: '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("Unterminated string at position {0}")]
    UnterminatedString(usize),
    #[error("Invalid number: '{0}' at position {1}")]
    InvalidNumber(String, usize),
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    
    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_next() == Some('/') {
                // Skip line comments
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> Result<String, LexerError> {
        let start_pos = self.pos;
        self.advance(); // Skip opening quote
        
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.peek() {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        _ => result.push(escaped),
                    }
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }
        Err(LexerError::UnterminatedString(start_pos))
    }
    
    fn read_number(&mut self) -> Result<f64, LexerError> {
        let start_pos = self.pos;
        let mut num_str = String::new();
        
        // Handle negative numbers
        if self.peek() == Some('-') {
            num_str.push(self.advance().unwrap());
        }
        
        // Read integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        // Read decimal part
        if self.peek() == Some('.') {
            num_str.push(self.advance().unwrap());
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        num_str
            .parse::<f64>()
            .map_err(|_| LexerError::InvalidNumber(num_str, start_pos))
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '/' || ch == '.' {
                result.push(ch);
                self.advance();
            } else if ch == ':' {
                // Check if this is a namespace separator (minecraft:stone) or a colon (to:)
                // If there's more text after :, it's a namespace
                // If : is at the end or followed by whitespace, it's a colon
                let next_pos = self.pos + 1;
                if let Some(next_ch) = self.input.get(next_pos) {
                    if next_ch.is_alphanumeric() || *next_ch == '_' || *next_ch == '/' || *next_ch == '.' {
                        // This is a namespace separator, include it
                        result.push(ch);
                        self.advance();
                    } else {
                        // This is a colon, stop here
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Ok(Token::EOF);
        }
        
        let ch = self.peek().unwrap();
        let start_pos = self.pos;
        
        // Check for special prefixes first
        if ch == '@' {
            self.advance();
            if let Some(next) = self.peek() {
                if next == '~' {
                    self.advance();
                    return Ok(Token::RelativeCoord);
                } else if next.is_alphabetic() {
                    // Entity selector like @a, @p, @e, @s, @r
                    let selector = self.read_identifier();
                    return Ok(Token::EntitySelector(format!("@{}", selector)));
                }
            }
            return Ok(Token::At);
        }
        
        if ch == '%' {
            self.advance();
            if let Some(next) = self.peek() {
                if next == '^' {
                    self.advance();
                    return Ok(Token::LocalCoord);
                }
            }
            return Ok(Token::Percent);
        }
        
        if ch == '#' {
            self.advance();
            return Ok(Token::Hash);
        }
        
        if ch == '$' {
            self.advance();
            let tag = self.read_identifier();
            return Ok(Token::FunctionTag(tag));
        }
        
        // Symbols
        match ch {
            '(' => { self.advance(); return Ok(Token::LParen); }
            ')' => { self.advance(); return Ok(Token::RParen); }
            '{' => { self.advance(); return Ok(Token::LBrace); }
            '}' => { self.advance(); return Ok(Token::RBrace); }
            '[' => { self.advance(); return Ok(Token::LBracket); }
            ']' => { self.advance(); return Ok(Token::RBracket); }
            ':' => { self.advance(); return Ok(Token::Colon); }
            ',' => { self.advance(); return Ok(Token::Comma); }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    return Ok(Token::EqEq);
                }
                return Ok(Token::Equals); // Single = for selectors and assignments
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    return Ok(Token::NotEq);
                }
                return Err(LexerError::UnexpectedChar('!', start_pos));
            }
            '-' => {
                self.advance();
                return Ok(Token::Minus);
            }
            '~' => {
                self.advance();
                return Ok(Token::Tilde);
            }
            '^' => {
                self.advance();
                return Ok(Token::Caret);
            }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    return Ok(Token::DotDot);
                }
                return Ok(Token::Dot);
            }
            '*' => {
                self.advance();
                return Ok(Token::Star);
            }
            '"' => {
                let s = self.read_string()?;
                return Ok(Token::String(s));
            }
            _ if ch.is_ascii_digit() || (ch == '-' && self.peek_next().map(|c| c.is_ascii_digit()).unwrap_or(false)) => {
                let num = self.read_number()?;
                return Ok(Token::Number(num));
            }
            _ if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                let token = match ident.as_str() {
                    "func" => Token::Func,
                    "if" => Token::If,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Identifier(ident),
                };
                return Ok(token);
            }
            _ => return Err(LexerError::UnexpectedChar(ch, start_pos)),
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let input = r#"#say "Hello""#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Hash);
        assert_eq!(tokens[1], Token::Identifier("say".to_string()));
        assert_eq!(tokens[2], Token::String("Hello".to_string()));
    }
    
    #[test]
    fn test_entity_selector() {
        let input = "@a";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::EntitySelector("@a".to_string()));
    }
    
    #[test]
    fn test_relative_coord() {
        let input = "@~";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::RelativeCoord);
    }
}
