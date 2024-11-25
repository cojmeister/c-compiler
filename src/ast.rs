use crate::scan::{Token, TokenError};
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub struct ASTNode {
    pub operation: Token,
    pub(crate) left: Option<Box<ASTNode>>,
    pub(crate) right: Option<Box<ASTNode>>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

    /// Make a leaf node (end node)
    ///
    /// # Arguments
    ///
    /// * `operation`: Has to be [`crate::scan::Token::INT`] otherwise will return [ASTError]
    ///
    /// returns: Result<ASTNode, ASTError>
    ///
    /// # Examples
    /// Given a non int token will return Err
    /// ```
    /// let leaf = crate::ast::ASTNode::make_leaf(Token::SLASH);
    /// assert_eq!(leaf, Err(ASTError::InvalidLeafNode));
    /// ```
    ///
    /// Otherwise, it will return an Ok node
    /// ```
    /// let leaf = ASTNode::make_leaf(Token::INT(4));
    /// assert_eq!(leaf, Ok(ASTNode {
    ///     operation: Token::INT(4),
    ///     left: None,
    ///     right: None,
    /// }));
    ///
    /// ```
    fn make_leaf(operation: Token) -> Result<Self, ASTError> {
        match operation {
            Token::INT(_) => Ok(Self {
                operation,
                left: None,
                right: None,
            }),
            _ => Err(ASTError::InvalidLeafNode),
        }
    }

    /// Gets operator precedence - higher means higher precedence
    fn get_precedence(token: &Token) -> Result<u8, ASTError> {
        match token {
            Token::PLUS | Token::MINUS => Ok(1),
            Token::ASTERISK | Token::SLASH => Ok(2),
            Token::EndOfLine | Token::EndOfFile => Err(ASTError::ExpectedOperator),
            _ => Ok(0),
        }
    }

    /// Parse a primary factor (numbers or parenthesized expressions)
    ///
    /// # Arguments
    ///
    /// * `tokens`: a vector of Result<[crate::Token], [crate::TokenError]>
    ///
    /// returns: Result<ASTNode, ASTError>
    fn parse_primary(tokens: &mut Peekable<IntoIter<Result<Token, TokenError>>>) -> Result<Self, ASTError> {
        match tokens.next() {
            Some(Ok(Token::INT(n))) => Self::make_leaf(Token::INT(n)),
            Some(Ok(token)) => Err(ASTError::UnexpectedToken(token)),
            Some(Err(token_error)) => Err(ASTError::LexicalError(token_error)),
            None => Err(ASTError::EmptyExpression),
        }
    }


    /// Parse a binary expression with operator precedence
    ///
    /// # Arguments
    ///
    /// * `tokens`: a peekable iterable of results of the token, as gotten from the scanner [`crate::scan_file`]
    /// * `min_precedence`: to be set to 0 in the call
    ///
    /// returns: Result<ASTNode, ASTError>
    fn parse_one_line_expression(
        tokens: &mut Peekable<IntoIter<Result<Token, TokenError>>>,
        min_precedence: u8,
    ) -> Result<Self, ASTError> {
        let mut left: ASTNode = Self::parse_primary(tokens)?;

        while let Some(Ok(op)) = tokens.peek().cloned() {
            let precedence = match Self::get_precedence(&op) {
                Ok(precedence) => precedence,
                Err(_) => break,
            };

            if precedence < min_precedence {
                break;
            }

            // Consume the operator, handling potential errors
            match tokens.next() {
                Some(Ok(_)) => (), // We already know it's valid from the peek
                Some(Err(err)) => return Err(ASTError::LexicalError(err)),
                None => return Err(ASTError::ExpectedOperator),
            }

            let right: ASTNode = Self::parse_one_line_expression(tokens, precedence + 1)?;
            left = Self::new(Ok(op), Box::new(left), Box::new(right))?;
        }

        // Handle error token if present
        if let Some(Err(err)) = tokens.peek() {
            return Err(ASTError::LexicalError(err.clone()));
        }

        Ok(left)
    }

    /// Main entry point for parsing
    ///
    /// # Arguments
    ///
    /// * `tokens`: a vector of token results, as received from the scanner
    ///
    /// returns: Result<ASTNode, ASTError>
    ///
    /// # Examples
    ///
    /// ```
    /// let tokens = vec![
    ///     Ok(Token::INT(5)),
    ///     Ok(Token::PLUS),
    ///     Ok(Token::INT(5)),
    /// ];
    /// let ast = ASTNode::parse(tokens).unwrap();
    /// assert_eq!(ast.test_evaluate().unwrap(), 10);
    /// ```
    pub fn parse(tokens: Vec<Result<Token, TokenError>>) -> Result<Self, ASTError> {
        if tokens.is_empty() {
            return Err(ASTError::EmptyExpression);
        }

        let mut token_iter = tokens.into_iter().peekable();
        Self::parse_one_line_expression(&mut token_iter, 0)
    }

    /// ## *For testing only!*
    /// Helper method to test_evaluate the AST (for testing)
    /// Will test_evaluate the AST
    fn test_evaluate(&self) -> Result<i32, ASTError> {
        match &self.operation {
            Token::INT(n) => Ok(*n),
            Token::PLUS => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                Ok(left + right)
            }
            Token::MINUS => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                Ok(left - right)
            }
            Token::ASTERISK => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                Ok(left * right)
            }
            Token::SLASH => {
                let left = self.left.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
                let right = self.right.as_ref().ok_or(ASTError::ExpectedInteger)?.test_evaluate()?;
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
    fn test_get_precedence_enf_of_line() {
        let token = Token::EndOfLine;
        let err = ASTNode::get_precedence(&token).err().unwrap();
        assert_eq!(err, ASTError::ExpectedOperator);
    }
    
    #[test]
    fn test_get_precedence_enf_of_file() {
        let token = Token::INT(0);
        let actual_precedence = ASTNode::get_precedence(&token).unwrap();
        assert_eq!(actual_precedence, 0);
    }

    #[test]
    fn test_simple_addition() {
        let tokens = vec![
            Ok(Token::INT(5)),
            Ok(Token::PLUS),
            Ok(Token::INT(3)),
        ];

        let ast = ASTNode::parse(tokens).unwrap();
        assert_eq!(ast.test_evaluate().unwrap(), 8);
    }
    
    #[test]
    fn test_simple_subtraction() {
        let tokens = vec![
            Ok(Token::INT(5)),
            Ok(Token::MINUS),
            Ok(Token::INT(3)),
        ];

        let ast = ASTNode::parse(tokens).unwrap();
        assert_eq!(ast.test_evaluate().unwrap(), 2);
    }
    
    #[test]
    fn test_simple_division() {
        let tokens = vec![
            Ok(Token::INT(9)),
            Ok(Token::SLASH),
            Ok(Token::INT(3)),
        ];

        let ast = ASTNode::parse(tokens).unwrap();
        assert_eq!(ast.test_evaluate().unwrap(), 3);
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
        assert_eq!(ast.test_evaluate().unwrap(), 14);
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

    #[test]
    fn test_make_leaf_returns_error() {
        let token = Token::SLASH;
        let actual = ASTNode::make_leaf(token);
        assert_eq!(actual, Err(ASTError::InvalidLeafNode));
    }

    #[test]
    fn test_make_leaf_returns_leaf() {
        let leaf = ASTNode::make_leaf(Token::INT(4));
        assert_eq!(leaf, Ok(ASTNode {
            operation: Token::INT(4),
            left: None,
            right: None,
        }));
    }
}