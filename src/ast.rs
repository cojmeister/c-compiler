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
    fn get_precedence(token: &Token) -> u8 {
        match token {
            Token::PLUS | Token::MINUS => 1,
            Token::ASTERISK | Token::SLASH => 2,
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

        while let Some(Ok(op)) = tokens.peek().cloned() {
            let precedence = Self::get_precedence(&op);

            if precedence < min_precedence {
                break;
            }

            // Consume the operator, handling potential errors
            match tokens.next() {
                Some(Ok(_)) => (), // We already know it's valid from the peek
                Some(Err(err)) => return Err(ASTError::LexicalError(err)),
                None => return Err(ASTError::ExpectedOperator),
            }

            let right: ASTNode = Self::parse_expression(tokens, precedence + 1)?;
            left = Self::new(Ok(op), Box::new(left), Box::new(right))?;
        }

        // Handle error token if present
        if let Some(Err(err)) = tokens.peek() {
            return Err(ASTError::LexicalError(err.clone()));
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

    // Helper method to evaluate the AST (for testing)
    fn evaluate(&self) -> Result<i32, ASTError> {
        match &self.operation {
            Token::INT(n) => Ok(*n),
            Token::PLUS => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                Ok(left + right)
            }
            Token::MINUS => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                Ok(left - right)
            }
            Token::ASTERISK => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                Ok(left * right)
            }
            Token::SLASH => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.evaluate()?;
                if right == 0 {
                    Err(ASTError::ExpectedInteger) // Should be a division by zero error
                } else {
                    Ok(left / right)
                }
            }
            Token::EndOfFile => Err(ASTError::UnexpectedToken(Token::EndOfFile)),
            Token::EndOfLine => Err(ASTError::UnexpectedToken(Token::EndOfLine)),
        }
    }
}


// Updated tests to handle Results
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_addition() {
        let tokens = vec![
            Ok(Token::INT(5)),
            Ok(Token::PLUS),
            Ok(Token::INT(3)),
        ];

        let ast = ASTNode::parse(tokens).unwrap();
        assert_eq!(ast.evaluate().unwrap(), 8);
    }

    #[test]
    fn test_operator_precedence() {
        let tokens = vec![
            Ok(Token::INT(2)),
            Ok(Token::PLUS),
            Ok(Token::INT(3)),
            Ok(Token::ASTERISK),
            Ok(Token::INT(4)),
        ];

        let ast = ASTNode::parse(tokens).unwrap();
        assert_eq!(ast.evaluate().unwrap(), 14);
    }

    #[test]
    fn test_with_lexical_error() {
        let tokens = vec![
            Ok(Token::INT(5)),
            Ok(Token::PLUS),
            Err(TokenError {
                line: 1,
                column: 5,
                character: '@',
            }),
        ];

        assert!(matches!(ASTNode::parse(tokens), Err(ASTError::LexicalError(_))));
    }

    #[test]
    fn test_empty_input() {
        assert!(matches!(
            ASTNode::parse(vec![]),
            Err(ASTError::EmptyExpression)
        ));
    }

    #[test]
    fn test_invalid_expression() {
        let tokens = vec![
            Ok(Token::PLUS),
            Ok(Token::INT(5)),
        ];
        assert!(matches!(ASTNode::parse(tokens), Err(_)));
    }
}