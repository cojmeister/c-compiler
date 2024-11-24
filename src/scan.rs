use std::io;
use std::io::BufRead;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    PLUS,
    MINUS,
    ASTERISK,
    SLASH,
    INT(i32),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TokenError {
    line: usize,
    column: usize,
    character: char,
}


/// Scan a file and return a vector of Tokens
///
/// # Arguments
///
/// * `reader`: a bufreader for the file we want to scan.
///
/// returns: Result<Vec<Result<Token, TokenError>, Global>, Error>
pub fn scan_file<R: BufRead>(reader: &mut R) -> io::Result<Vec<Result<Token, TokenError>>> {
    let mut tokens: Vec<Result<Token, TokenError>> = Vec::new();
    let mut line_num: usize = 1;

    for line in reader.lines() {
        let line = line?;
        let line_tokens = scan_line(&line, line_num);
        tokens.extend(line_tokens);
        line_num += 1;
    }

    Ok(tokens)
}


/// Scan a single line in a file and return a vector of tokens, wrapped in result.
///
/// # Arguments
///
/// * `line`: The line that will be scanned
/// * `line_num`: the line number of the given line, for debugging purposes
///
/// returns: Vec<Result<Token, TokenError>, Global>
///
/// # Examples
///
/// ```
/// let line: &str = "10 + 9 * 6";
/// let tokens = scan_line(line, 0);
/// assert!(tokens.len() == 5);
/// ```
pub fn scan_line(line: &str, line_num: usize) -> Vec<Result<Token, TokenError>> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().enumerate().peekable();

    while let Some((column, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        let token_result = scan_token(ch, &mut chars, line_num, column);
        tokens.push(token_result);
    }
    tokens
}


/// Function to scan a single token
///
/// # Arguments
///
/// * `current_char`: the current character in the line
/// * `chars`: a peekable iterator for the current line (allows to parse numbers)
/// * `line`: line number, for debugging
/// * `column`: column  number for debugging
///
/// returns: Result<Token, TokenError>
///
/// # Examples
///
/// ```
/// let mut peekable_chars = "-123".chars().peekable();
/// let current_char = '-';
/// let token = scan_token(current_char, peekable_chars, 0, 0).unwrap();
/// assert!(token== Token::MINUS);
/// ```
pub fn scan_token(
    current_char: char,
    chars: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Chars>>,
    line: usize,
    column: usize,
) -> Result<Token, TokenError> {
    match current_char {
        '+' => Ok(Token::PLUS),
        '-' => Ok(Token::MINUS),
        '*' => Ok(Token::ASTERISK),
        '/' => Ok(Token::SLASH),
        '0'..='9' => {
            let mut number = current_char.to_string();

            // Using peek() to look ahead without consuming
            while let Some(&(_, next_char)) = chars.peek() {
                if !next_char.is_digit(10) {
                    break;
                }
                number.push(next_char);
                chars.next(); // Now consume the character we peeked at
            }

            match number.parse::<i32>() {
                Ok(num) => Ok(Token::INT(num)),
                Err(_) => Err(TokenError {
                    line,
                    column,
                    character: current_char,
                }),
            }
        }
        _ => Err(TokenError {
            line,
            column,
            character: current_char,
        }),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper function to create a BufReader from a string
    fn create_reader(input: &str) -> Cursor<Vec<u8>> {
        Cursor::new(input.as_bytes().to_vec())
    }

    #[test]
    fn test_scan_token_operators() {
        let mut chars = "".chars().enumerate().peekable();

        // Test basic operators
        assert!(matches!(
            scan_token('+', &mut chars, 1, 0),
            Ok(Token::PLUS)
        ));
        assert!(matches!(
            scan_token('-', &mut chars, 1, 0),
            Ok(Token::MINUS)
        ));
        assert!(matches!(
            scan_token('*', &mut chars, 1, 0),
            Ok(Token::ASTERISK)
        ));
        assert!(matches!(
            scan_token('/', &mut chars, 1, 0),
            Ok(Token::SLASH)
        ));
    }

    #[test]
    fn test_scan_token_integers() {
        // Test single digit
        let mut chars = "".chars().enumerate().peekable();
        if let Ok(Token::INT(value)) = scan_token('5', &mut chars, 1, 0) {
            assert_eq!(value, 5);
        } else {
            panic!("Failed to parse single digit integer");
        }

        // Test multi-digit number
        let mut chars = "23".chars().enumerate().peekable();
        if let Ok(Token::INT(value)) = scan_token('1', &mut chars, 1, 0) {
            assert_eq!(value, 123);
        } else {
            panic!("Failed to parse multi-digit integer");
        }

        // Test multi-digit number again
        let mut chars = "030432".chars().enumerate().peekable();
        if let Ok(Token::INT(value)) = scan_token('9', &mut chars, 1, 0) {
            assert_eq!(value, 9030432);
        } else {
            panic!("Failed to parse multi-digit integer");
        }
    }

    #[test]
    fn scan_token_int() {
        // Test multi-digit number
        let mut chars = "023".chars().enumerate().peekable();
        let token = scan_token('1', &mut chars, 1, 0);
        assert_eq!(token.unwrap(), Token::INT(1023));
    }

    #[test]
    fn test_scan_token_invalid_char() {
        let mut chars = "".chars().enumerate().peekable();
        if let Err(error) = scan_token('@', &mut chars, 1, 5) {
            assert_eq!(error.line, 1);
            assert_eq!(error.column, 5);
            assert_eq!(error.character, '@');
        } else {
            panic!("Invalid character was accepted");
        }
    }

    #[test]
    fn test_scan_line_simple() {
        let tokens = scan_line("1 + 2", 1);
        assert_eq!(tokens.len(), 3);

        assert!(matches!(tokens[0], Ok(Token::INT(1))));
        assert!(matches!(tokens[1], Ok(Token::PLUS)));
        assert!(matches!(tokens[2], Ok(Token::INT(2))));
    }

    #[test]
    fn test_scan_line_with_whitespace() {
        let tokens = scan_line("   42    *    5   ", 1);
        assert_eq!(tokens.len(), 3);

        assert!(matches!(tokens[0], Ok(Token::INT(42))));
        assert!(matches!(tokens[1], Ok(Token::ASTERISK)));
        assert!(matches!(tokens[2], Ok(Token::INT(5))));
    }

    #[test]
    fn test_scan_line_many_operations() {
        let tokens = scan_line("1+3-5*44/6+4", 1);
        assert_eq!(tokens.len(), 11);

        assert!(matches!(tokens[0], Ok(Token::INT(1))));
        assert!(matches!(tokens[1], Ok(Token::PLUS)));
        assert!(matches!(tokens[2], Ok(Token::INT(3))));
        assert!(matches!(tokens[3], Ok(Token::MINUS)));
        assert!(matches!(tokens[4], Ok(Token::INT(5))));
        assert!(matches!(tokens[5], Ok(Token::ASTERISK)));
        assert!(matches!(tokens[6], Ok(Token::INT(44))));
        assert!(matches!(tokens[7], Ok(Token::SLASH)));
        assert!(matches!(tokens[8], Ok(Token::INT(6))));
        assert!(matches!(tokens[9], Ok(Token::PLUS)));
        assert!(matches!(tokens[10], Ok(Token::INT(4))));
    }

    #[test]
    fn test_scan_line_with_error() {
        let tokens = scan_line("1 @ 2", 1);
        assert_eq!(tokens.len(), 3);

        assert!(matches!(tokens[0], Ok(Token::INT(1))));
        assert_eq!(
            tokens[1].clone().err().unwrap(),
            TokenError { line: 1, column: 2, character: '@' }
        );
        assert!(matches!(tokens[2], Ok(Token::INT(2))));
    }

    #[test]
    fn test_scan_file() {
        let input = "1 + 2\n3 * 4\n";
        let mut reader = create_reader(input);

        let result = scan_file(&mut reader).unwrap();
        assert_eq!(result.len(), 6);

        // First line
        assert!(matches!(result[0], Ok(Token::INT(1))));
        assert!(matches!(result[1], Ok(Token::PLUS)));
        assert!(matches!(result[2], Ok(Token::INT(2))));

        // Second line
        assert!(matches!(result[3], Ok(Token::INT(3))));
        assert!(matches!(result[4], Ok(Token::ASTERISK)));
        assert!(matches!(result[5], Ok(Token::INT(4))));
    }

    #[test]
    fn test_scan_file_with_errors() {
        let input = "1 + 2\n3   @ 4\n";
        let mut reader = create_reader(input);

        let result = scan_file(&mut reader).unwrap();

        // Check first line - should be all OK
        assert!(matches!(result[0], Ok(Token::INT(1))));
        assert!(matches!(result[1], Ok(Token::PLUS)));
        assert!(matches!(result[2], Ok(Token::INT(2))));

        // Check second line - should have an error
        assert!(matches!(result[3], Ok(Token::INT(3))));
        assert_eq!(
            result[4].clone().err().unwrap(),
            TokenError { line: 2, column: 4, character: '@' }
        );
        assert!(matches!(result[5], Ok(Token::INT(4))));
    }

    #[test]
    fn test_empty_file() {
        let input = "";
        let mut reader = create_reader(input);

        let result = scan_file(&mut reader).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_lines() {
        let input = "\n\n\n";
        let mut reader = create_reader(input);

        let result = scan_file(&mut reader).unwrap();
        assert!(result.is_empty());
    }
}