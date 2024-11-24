use crate::scan::{Token, TokenError};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct ASTNode {
    operation: Token,
    left: Option<Box<ASTNode>>,
    right: Option<Box<ASTNode>>,
}

#[derive(Debug)]
pub enum ASTError {
    UnexpectedToken(Token),
    LexicalError(TokenError),
    ExpectedOperator,
    ExpectedInteger,
    EmptyExpression,
    InvalidLeafNode,
}

impl ASTNode {
    pub fn new(operation: Result<Token, TokenError>, left: Box<ASTNode>, right: Box<ASTNode>) -> Result<Self, ASTError> {
        match operation {
            Ok(op) => Ok(Self {
                operation: op,
                left: Some(left),
                right: Some(right),
            }),
            Err(token) => Err(ASTError::LexicalError(token)),
        }
    }

    pub fn make_leaf(operation: Token) -> Result<Self, ASTError> {
        match operation {
            Token::INT(_) => Ok(Self {
                operation,
                left: None,
                right: None,
            }),
            _ => Err(ASTError::InvalidLeafNode),
        }
    }

    pub fn make_unary(operation: Token, left: Box<ASTNode>) -> Self {
        Self {
            operation,
            left: Some(left),
            right: None,
        }
    }

    // Gets operator precedence - higher means higher precedence
    fn get_precedence(token: &Result<Token, TokenError>) -> u8 {
        match token {
            Ok(Token::PLUS) | Ok(Token::MINUS) => 1,
            Ok(Token::ASTERISK) | Ok(Token::SLASH) => 2,
            _ => 0,
        }
    }

    // Parse a primary factor (numbers or parenthesized expressions)
    fn parse_primary(tokens: &mut Peekable<IntoIter<Result<Token, TokenError>>>) -> Result<Self, ASTError> {
        match tokens.next() {
            Some(Ok(Token::INT(n))) => Self::make_leaf(Token::INT(n)),
            Some(Ok(token)) => Err(ASTError::UnexpectedToken(token)),
            Some(Err(token_error)) => Err(ASTError::LexicalError(token_error)),
            None => Err(ASTError::EmptyExpression),
        }
    }


    // Parse a binary expression with operator precedence
    fn parse_expression(
        tokens: &mut Peekable<IntoIter<Result<Token, TokenError>>>,
        min_precedence: u8,
    ) -> Result<Self, ASTError> {
        let mut left: ASTNode = Self::parse_primary(tokens)?;

        while let Some(op) = tokens.peek() {
            let op = op.clone();
            let precedence = Self::get_precedence(&op);

            if precedence < min_precedence {
                break;
            }

            tokens.next(); // Consume the operator

            let right = Self::parse_expression(tokens, precedence + 1)?;

            left = Self::new(op, Box::new(left), Box::new(right))?;
        }

        Ok(left)
    }

    // Main entry point for parsing
    pub fn parse(tokens: Vec<Result<Token, TokenError>>) -> Result<Self, ASTError> {
        if tokens.is_empty() {
            return Err(ASTError::EmptyExpression);
        }

        let mut token_iter = tokens.into_iter().peekable();
        Self::parse_expression(&mut token_iter, 0)
    }


}