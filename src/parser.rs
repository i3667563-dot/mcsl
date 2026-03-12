//! Parser for MCSL - converts tokens to AST

use crate::ast::*;
use crate::lexer::{LexerError, Token};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected token: {0}, expected: {1}")]
    UnexpectedToken(String, String),
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexerError),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParserError> {
        let token = self.advance();
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken(
                format!("{:?}", token),
                format!("{:?}", expected),
            ))
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParserError> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            token => Err(ParserError::UnexpectedToken(
                format!("{:?}", token),
                "Identifier".to_string(),
            )),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        match self.peek().clone() {
            Token::String(s) => {
                self.advance();
                Ok(Expr::String(s))
            }
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::Bool(b) => {
                self.advance();
                Ok(Expr::Bool(b))
            }
            Token::Star => {
                self.advance();
                Ok(Expr::String("*".to_string()))
            }
            Token::EntitySelector(sel) => {
                self.advance();
                // Check for selector arguments [@a[tag=test], etc.]
                if self.peek() == &Token::LBracket {
                    self.advance(); // [
                    let args = self.parse_selector_args()?;
                    self.expect(Token::RBracket)?;
                    Ok(Expr::SpecialArg(SpecialArg::EntitySelector(format!(
                        "{}{}",
                        sel, args
                    ))))
                } else {
                    Ok(Expr::SpecialArg(SpecialArg::EntitySelector(sel)))
                }
            }
            Token::RelativeCoord => {
                self.advance();
                let offset = if self.peek() == &Token::LParen {
                    self.advance(); // (
                    let val = match self.advance() {
                        Token::Number(n) => Some(n),
                        Token::Minus => {
                            if let Token::Number(n) = self.advance() {
                                Some(-n)
                            } else {
                                return Err(ParserError::UnexpectedEOF);
                            }
                        }
                        _ => None,
                    };
                    self.expect(Token::RParen)?;
                    val
                } else {
                    None
                };
                Ok(Expr::Coords(Coords {
                    x: CoordValue::Relative(offset),
                    y: CoordValue::Relative(None),
                    z: CoordValue::Relative(None),
                }))
            }
            Token::LBracket => {
                self.advance(); // [
                let exprs = self.parse_array()?;
                self.expect(Token::RBracket)?;
                Ok(Expr::Array(exprs))
            }
            Token::At => {
                // Standalone @ - might be part of coordinate array
                self.advance();
                Ok(Expr::SpecialArg(SpecialArg::EntitySelector(
                    "@".to_string(),
                )))
            }
            token => Err(ParserError::UnexpectedToken(
                format!("{:?}", token),
                "Expression".to_string(),
            )),
        }
    }

    fn parse_selector_args(&mut self) -> Result<String, ParserError> {
        let mut args = Vec::new();
        loop {
            let key = self.parse_identifier()?;

            // Entity selectors use = instead of :
            // Check for either = or :
            match self.advance() {
                Token::Colon | Token::Equals => {}
                token => {
                    return Err(ParserError::UnexpectedToken(
                        format!("{:?}", token),
                        " '=' or ':'".to_string(),
                    ))
                }
            }

            // Handle range values like ..10 or 5..10
            let mut value = String::new();

            // Check for .. prefix (e.g., ..10)
            if self.peek() == &Token::DotDot {
                self.advance();
                value.push_str("..");
            }

            let value_part = match self.advance() {
                Token::Identifier(s) => s,
                Token::String(s) => s,
                Token::Number(n) => n.to_string(),
                token => {
                    return Err(ParserError::UnexpectedToken(
                        format!("{:?}", token),
                        "Selector value".to_string(),
                    ))
                }
            };
            value.push_str(&value_part);

            // Check for .. suffix (e.g., 5..10)
            if self.peek() == &Token::DotDot {
                self.advance();
                value.push_str("..");
                if let Token::Number(n) = self.advance() {
                    value.push_str(&n.to_string());
                }
            }

            args.push(format!("{}={}", key, value));

            if self.peek() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        Ok(format!("[{}]", args.join(",")))
    }

    fn parse_array(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut exprs = Vec::new();

        // Check for coordinate array [@~ @~ @~] or [@~ @~(1) @~]
        if self.peek() == &Token::RelativeCoord || self.peek() == &Token::LocalCoord {
            let mut coords = Vec::new();
            while self.peek() != &Token::RBracket && self.peek() != &Token::EOF {
                let coord = self.parse_coord_value()?;
                coords.push(coord);

                if self.peek() == &Token::Comma {
                    self.advance();
                }
            }

            if coords.len() == 3 {
                return Ok(vec![Expr::Coords(Coords {
                    x: coords[0].clone(),
                    y: coords[1].clone(),
                    z: coords[2].clone(),
                })]);
            }
        }

        // Regular array - parse expressions
        while self.peek() != &Token::RBracket && self.peek() != &Token::EOF {
            // Check if this is a named argument inside an array (like to: [...])
            // If so, stop parsing this array and let the parent handle it
            if let Token::Identifier(_) = self.peek().clone() {
                let next_pos = self.pos + 1;
                if self.tokens.get(next_pos) == Some(&Token::Colon) {
                    // This is a named argument, stop parsing array
                    break;
                }
            }

            let expr = self.parse_expr()?;
            exprs.push(expr);

            if self.peek() == &Token::Comma {
                self.advance();
            }
        }

        Ok(exprs)
    }

    fn parse_coord_value(&mut self) -> Result<CoordValue, ParserError> {
        match self.peek().clone() {
            Token::RelativeCoord => {
                self.advance();
                let offset = if self.peek() == &Token::LParen {
                    self.advance(); // (
                    let val = match self.advance() {
                        Token::Number(n) => Some(n),
                        token => {
                            return Err(ParserError::UnexpectedToken(
                                format!("{:?}", token),
                                "Number".to_string(),
                            ))
                        }
                    };
                    self.expect(Token::RParen)?;
                    val
                } else {
                    None
                };
                Ok(CoordValue::Relative(offset))
            }
            Token::LocalCoord => {
                self.advance();
                let offset = if self.peek() == &Token::LParen {
                    self.advance(); // (
                    let val = match self.advance() {
                        Token::Number(n) => Some(n),
                        token => {
                            return Err(ParserError::UnexpectedToken(
                                format!("{:?}", token),
                                "Number".to_string(),
                            ))
                        }
                    };
                    self.expect(Token::RParen)?;
                    val
                } else {
                    None
                };
                Ok(CoordValue::Local(offset))
            }
            Token::Number(n) => {
                self.advance();
                Ok(CoordValue::Absolute(n))
            }
            token => Err(ParserError::UnexpectedToken(
                format!("{:?}", token),
                "Coordinate value".to_string(),
            )),
        }
    }

    fn parse_command_args(&mut self) -> Result<Vec<CommandArg>, ParserError> {
        let mut args = Vec::new();

        while self.peek() != &Token::RParen && self.peek() != &Token::EOF {
            // Check for named argument: name: value
            if let Token::Identifier(name) = self.peek().clone() {
                let next_pos = self.pos + 1;
                if self.tokens.get(next_pos) == Some(&Token::Colon) {
                    self.advance(); // name
                    self.advance(); // :
                    let value = self.parse_expr()?;
                    args.push(CommandArg::Named(name, value));

                    if self.peek() == &Token::Comma {
                        self.advance();
                    }
                    continue;
                }
            }

            // Positional argument
            let expr = self.parse_expr()?;
            args.push(CommandArg::Positional(expr));

            if self.peek() == &Token::Comma {
                self.advance();
            }
        }

        Ok(args)
    }

    fn parse_block(&mut self) -> Result<Block, ParserError> {
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();

        while self.peek() != &Token::RBrace && self.peek() != &Token::EOF {
            statements.push(self.parse_statement()?);
        }

        self.expect(Token::RBrace)?;
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek().clone() {
            Token::Hash => {
                self.advance(); // #

                // Check for #run specifically - calls a function
                if let Token::Identifier(name) = self.peek().clone() {
                    if name == "run" {
                        self.advance(); // run
                        let func_name = self.parse_identifier()?;
                        return Ok(Statement::FunctionCall(func_name));
                    }
                }

                let cmd_name = self.parse_identifier()?;

                // #command (args) or #command args...
                let args = if self.peek() == &Token::LParen {
                    self.advance(); // (
                    let args = self.parse_command_args()?;
                    self.expect(Token::RParen)?;
                    args
                } else {
                    // Simple command without parentheses - collect tokens until end of line or next special token
                    let mut args = Vec::new();
                    while self.peek() != &Token::EOF
                        && !matches!(self.peek(), Token::RBrace | Token::Hash)
                    {
                        match self.advance() {
                            Token::Identifier(s) => {
                                args.push(CommandArg::Positional(Expr::String(s)))
                            }
                            Token::String(s) => args.push(CommandArg::Positional(Expr::String(s))),
                            Token::Number(n) => args.push(CommandArg::Positional(Expr::Number(n))),
                            Token::EntitySelector(sel) => {
                                // Check for selector arguments like @e[type=zombie]
                                if self.peek() == &Token::LBracket {
                                    self.advance(); // [
                                    let selector_args = self.parse_selector_args()?;
                                    self.expect(Token::RBracket)?;
                                    args.push(CommandArg::Positional(Expr::SpecialArg(
                                        SpecialArg::EntitySelector(format!(
                                            "{}{}",
                                            sel, selector_args
                                        )),
                                    )))
                                } else {
                                    args.push(CommandArg::Positional(Expr::SpecialArg(
                                        SpecialArg::EntitySelector(sel),
                                    )))
                                }
                            }
                            Token::Bool(b) => args.push(CommandArg::Positional(Expr::Bool(b))),
                            Token::LBracket => {
                                // Parse array
                                let mut arr = Vec::new();
                                while self.peek() != &Token::RBracket && self.peek() != &Token::EOF
                                {
                                    let expr = self.parse_expr()?;
                                    arr.push(expr);
                                    if self.peek() == &Token::Comma {
                                        self.advance();
                                    }
                                }
                                self.expect(Token::RBracket)?;
                                args.push(CommandArg::Positional(Expr::Array(arr)));
                            }
                            Token::Tilde => {
                                // Handle ~ coordinate
                                let offset = if matches!(self.peek(), Token::Number(_)) {
                                    if let Token::Number(n) = self.advance() {
                                        Some(n)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                args.push(CommandArg::Positional(Expr::Coords(Coords {
                                    x: CoordValue::Relative(offset),
                                    y: CoordValue::Relative(None),
                                    z: CoordValue::Relative(None),
                                })));
                            }
                            Token::RelativeCoord => {
                                // Handle @~ coordinate
                                let offset = if matches!(self.peek(), Token::Number(_)) {
                                    if let Token::Number(n) = self.advance() {
                                        Some(n)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                args.push(CommandArg::Positional(Expr::Coords(Coords {
                                    x: CoordValue::Relative(offset),
                                    y: CoordValue::Relative(None),
                                    z: CoordValue::Relative(None),
                                })));
                            }
                            Token::Minus => {
                                // Handle negative numbers
                                if let Token::Number(n) = self.advance() {
                                    args.push(CommandArg::Positional(Expr::Number(-n)));
                                }
                            }
                            // Skip whitespace tokens and continue
                            Token::Colon | Token::Comma => {}
                            _ => break,
                        }
                    }
                    args
                };
                Ok(Statement::Command(cmd_name, args))
            }
            Token::If => {
                self.advance(); // if
                self.expect(Token::LParen)?;

                // Parse condition: (target) (type) (operator)
                let target = self.parse_expr()?;

                // Skip to closing paren for now (simplified)
                while self.peek() != &Token::RParen && self.peek() != &Token::EOF {
                    self.advance();
                }
                self.expect(Token::RParen)?;

                let condition = IfCondition {
                    target,
                    check_type: "entity".to_string(), // Default
                    operator: "==".to_string(),
                };

                let body = self.parse_block()?;
                Ok(Statement::IfBlock(condition, body))
            }
            Token::Identifier(name) => {
                // Regular command without # (e.g., teleport(...), summon(...))
                let cmd_name = name;
                self.advance();

                // Expect parentheses for function-style commands
                if self.peek() != &Token::LParen {
                    return Err(ParserError::UnexpectedToken(
                        format!("{:?}", self.peek()),
                        "(".to_string(),
                    ));
                }

                self.advance(); // (
                let args = self.parse_command_args()?;
                self.expect(Token::RParen)?;

                Ok(Statement::Command(cmd_name, args))
            }
            token => Err(ParserError::UnexpectedToken(
                format!("{:?}", token),
                "Statement".to_string(),
            )),
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDef, ParserError> {
        let mut tag = None;

        // Check for $tag before func
        if let Token::FunctionTag(tag_name) = self.peek().clone() {
            self.advance();
            tag = Some(match tag_name.as_str() {
                "load" => FunctionTag::Load,
                "tick" => FunctionTag::Tick,
                _ => FunctionTag::Load, // Default
            });
        }

        self.expect(Token::Func)?;
        let name = self.parse_identifier()?;
        let body = self.parse_block()?;

        Ok(FunctionDef { name, tag, body })
    }

    fn parse_top_level(&mut self) -> Result<TopLevelItem, ParserError> {
        // Check for $tag func
        if let Token::FunctionTag(_) = self.peek().clone() {
            return Ok(TopLevelItem::Function(self.parse_function()?));
        }

        match self.peek().clone() {
            Token::Func => Ok(TopLevelItem::Function(self.parse_function()?)),
            Token::Hash => Ok(TopLevelItem::Statement(self.parse_statement()?)),
            Token::If => Ok(TopLevelItem::Statement(self.parse_statement()?)),
            Token::EOF => Err(ParserError::UnexpectedEOF),
            _ => {
                // Try to parse as statement
                Ok(TopLevelItem::Statement(self.parse_statement()?))
            }
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut program = Program::new();

        while self.peek() != &Token::EOF {
            let item = self.parse_top_level()?;
            program.items.push(item);
        }

        Ok(program)
    }
}
