use thiserror::Error;
// ========================================================================================
// ENUM
// ========================================================================================

#[derive(Debug, Error)]
pub enum TokenizerError {
    #[error("Unexpected character '{0}' at position {1}")]
    UnexpectedCharacter(char, usize),

    #[error("Unterminated string literal starting at position {0}")]
    UnterminatedString(usize),

    #[error("Invalid numeric literal '{0}' at position {1}")]
    InvalidNumeric(String, usize),

    #[error("Empty input provided")]
    EmptyInput,

    #[error("Invalid identifier '{0}' at position {1}")]
    InvalidIdentifier(String, usize),

    #[error("Unexpected end of input")]
    UnexpectedEof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Select,
    From,
    Where,
    Insert,
    Delete,
    Into,
    Values,

    // Identifiers and Literals
    Identifier(String),
    StringLiteral(String),
    NumericLiteral(String),
    

    // Symbols
    Semicolon,
    Asterisk,
    
    OpenBracket,
    CloseBracket,
    Comma,
    Index,
    Table,
    Database,
    
    // Binary Operators
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    And,
    Or,

    // DDL for Table and Constituents
    CreateTable,
    Create,
    Drop,
    Alter,

    // End of Input
    Eof,
}

// ========================================================================================
// STRUCT
// ========================================================================================

pub struct Tokenizer<'a> {
    input: &'a str,
    position: usize,
    ch: u8,
}

// ========================================================================================
// IMPLEMENTATION
// ========================================================================================
impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Self {
            input,
            position: 0,
            ch: 0,
        };
        // Get first to ensure correct pos. 
        tokenizer.read_char();
        tokenizer 
    }
    
    pub fn get_next_token(&mut self) -> Result<Token, TokenizerError> {
        self.skip_whitespace();

        let token = match self.ch {
            // Dont forget teh b is a byte literal
            b'=' => Ok(Token::Equals),
            b';' => Ok(Token::Semicolon),
            b'*' => Ok(Token::Asterisk),
            b'(' => Ok(Token::OpenBracket),
            b')' => Ok(Token::CloseBracket),
            b',' => Ok(Token::Comma),
            b'\'' => self.read_string_literal(),
            // This is the end of the input string.
            0 => Ok(Token::Eof),

            // Binary Operators longer than a single character
            b'>' => {
                if self.position < self.input.len() && self.input.as_bytes()[self.position] == b'=' {
                    self.read_char(); 
                    Ok(Token::GreaterThanOrEquals)
                } else {
                    Ok(Token::GreaterThan)
                }
            },
            b'<' => {
                if self.position < self.input.len() && self.input.as_bytes()[self.position] == b'=' {
                    self.read_char(); 
                    Ok(Token::LessThanOrEquals)
                } else {
                    Ok(Token::LessThan)
                }
            },
            b'!' => {
                if self.position < self.input.len() && self.input.as_bytes()[self.position] == b'=' {
                    self.read_char();
                    Ok(Token::NotEquals)
                } else {
                    Err(TokenizerError::UnexpectedCharacter(self.ch as char, self.position))
                }
            }


            // If it's a letter, it's either a keyword or an identifier.
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let literal = self.read_identifier();
                return Ok(Self::lookup_ident(&literal));
            }
            
            // If it's a digit, it's a number.
            b'0'..=b'9' => {
                let literal = self.read_numeric_literal();
                return Ok(Token::NumericLiteral(literal));
            },
            _ => Err(TokenizerError::UnexpectedCharacter(self.ch as char, self.position)),
        };

        self.read_char();
        token
    }

    fn read_char(&mut self) {
        if self.position >= self.input.len() {
            self.ch = 0; 
        } else {
            self.ch = self.input.as_bytes()[self.position];
        }
        self.position += 1;
    }
    
    fn read_identifier(&mut self) -> String {
        let start_pos = self.position - 1;
        while self.ch.is_ascii_alphanumeric() || self.ch == b'_' {
            self.read_char();
        }
        self.input[start_pos..self.position - 1].to_string()
    }
    
    fn read_numeric_literal(&mut self) -> String {
        let start_pos = self.position - 1;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        self.input[start_pos..self.position - 1].to_string()
    }

    fn read_string_literal(&mut self) -> Result<Token, TokenizerError> {
        let start_pos = self.position;
        self.read_char(); // Consume the opening quote
        
        while self.ch != b'\'' {
            if self.ch == 0 { // Reached end of input without closing quote
                return Err(TokenizerError::UnterminatedString(start_pos));
            }
            self.read_char();
        }

        let literal = self.input[start_pos..self.position-1].to_string();
        Ok(Token::StringLiteral(literal))
    }

    fn lookup_ident(ident: &str) -> Token {
        match ident.to_uppercase().as_str() {
            "SELECT" => Token::Select,
            "FROM" => Token::From,
            "INTO" => Token::Into,
            "WHERE" => Token::Where,
            "INSERT" => Token::Insert,
            "DELETE" => Token::Delete,
            "AND" => Token::And,
            "OR" => Token::Or,
            "VALUES" => Token::Values,
            _ => Token::Identifier(ident.to_string()),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }
}


// ========================================================================================
// TESTS
// ========================================================================================

#[cfg(test)]
mod tokenizer_tests {
    use super::*;

    #[test]
    fn test_select_with_wildcard_token() -> Result<(), TokenizerError> {
        let select_query = "SELECT * FROM table WHERE name = 'PHILIP';";
        let mut tokenizer = Tokenizer::new(select_query);

        let expected_tokens = vec![
            Token::Select,
            Token::Asterisk,
            Token::From,
            Token::Identifier("table".to_string()),
            Token::Where,
            Token::Identifier("name".to_string()),
            Token::Equals,
            Token::StringLiteral("PHILIP".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];
        
        let mut generated_tokens = Vec::new();
        loop {
            let token = tokenizer.get_next_token()?;
            let is_eof = token == Token::Eof;
            generated_tokens.push(token);
            if is_eof {
                break;
            }
        }

        assert_eq!(expected_tokens, generated_tokens);
        Ok(())
    }

    #[test]
    fn test_with_create_table() -> Result<(), TokenizerError> {
        let create_query = "CREATE TABLE new_table (column1 String, column2 String, column3 Integer);";
        let mut tokenizer = Tokenizer::new(create_query);

        let expected_tokens = vec![
            Token::Identifier("CREATE".to_string()),
            Token::Identifier("TABLE".to_string()),
            Token::Identifier("new_table".to_string()),
            Token::OpenBracket,
            Token::Identifier("column1".to_string()),
            Token::Identifier("String".to_string()),
            Token::Comma,
            Token::Identifier("column2".to_string()),
            Token::Identifier("String".to_string()),
            Token::Comma,
            Token::Identifier("column3".to_string()),
            Token::Identifier("Integer".to_string()),
            Token::CloseBracket,
            Token::Semicolon,
            Token::Eof,
        ];
        
        let mut generated_tokens = Vec::new();
        loop {
            let token = tokenizer.get_next_token()?;
            let is_eof = token == Token::Eof;
            generated_tokens.push(token);
            if is_eof {
                break;
            }
        }

        assert_eq!(expected_tokens, generated_tokens);
        Ok(())
    }

    #[test]
    fn test_case_insensitivity_and_identifiers() -> Result<(), TokenizerError> {
        let query = "SeLeCt Name FROM Users;";
        let mut tokenizer = Tokenizer::new(query);

        let expected_tokens = vec![
            Token::Select,
            Token::Identifier("Name".to_string()),   
            Token::From,
            Token::Identifier("Users".to_string()),  
            Token::Semicolon,
            Token::Eof,
        ];

        let mut generated_tokens = Vec::new();
        loop {
            let token = tokenizer.get_next_token()?;
            let is_eof = token == Token::Eof;
            generated_tokens.push(token);
            if is_eof {
                break;
            }
        }

        assert_eq!(expected_tokens, generated_tokens);
        Ok(())
    }

    #[test]
    fn test_binary_operators_and_keywords() -> Result<(), TokenizerError> {
        let query = "SELECT column1 FROM table WHERE value1 >= 10 AND value2 <= 20 OR value3 != 'test';";
        let mut tokenizer = Tokenizer::new(query);

        let expected_tokens = vec![
            Token::Select,
            Token::Identifier("column1".to_string()),
            Token::From,
            Token::Identifier("table".to_string()),
            Token::Where,
            Token::Identifier("value1".to_string()),
            Token::GreaterThanOrEquals,
            Token::NumericLiteral("10".to_string()),
            Token::And,
            Token::Identifier("value2".to_string()),
            Token::LessThanOrEquals,
            Token::NumericLiteral("20".to_string()),
            Token::Or,
            Token::Identifier("value3".to_string()),
            Token::NotEquals,
            Token::StringLiteral("test".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        let mut generated_tokens = Vec::new();
        loop {
            let token = tokenizer.get_next_token()?;
            let is_eof = token == Token::Eof;
            generated_tokens.push(token);
            if is_eof {
                break;
            }
        }

        assert_eq!(expected_tokens, generated_tokens);
        Ok(())
    }

    #[test]
    fn test_insert_statement() -> Result<(), TokenizerError> {
        let query = "INSERT INTO table VALUES ('first','second', '3', '4th');";
        let mut tokenizer = Tokenizer::new(query);

        let expected_tokens = vec![
            Token::Insert,
            Token::Into,
            Token::Identifier("table".to_string()),
            Token::Identifier("VALUES".to_string()),
            Token::OpenBracket,
            Token::StringLiteral("first".to_string()),
            Token::Comma,
            Token::StringLiteral("second".to_string()),
            Token::Comma,
            Token::StringLiteral("3".to_string()),
            Token::Comma,
            Token::StringLiteral("4th".to_string()),
            Token::CloseBracket,
            Token::Semicolon,
            Token::Eof,
        ];

        let mut generated_tokens = Vec::new();
        loop {
            let token = tokenizer.get_next_token()?;
            let is_eof = token == Token::Eof;
            generated_tokens.push(token);
            if is_eof {
            break;
            }
        }

        assert_eq!(expected_tokens, generated_tokens);
        Ok(())
    }

    #[test]
    fn test_insert_statement_with_columns() -> Result<(), TokenizerError> {
        let query = "INSERT INTO table (firstColumn, secondColumn, thirdColumn, fourthColumn) VALUES ('first','second', '3', '4th');";
        let mut tokenizer = Tokenizer::new(query);

        let expected_tokens = vec![
            Token::Insert,
            Token::Into,
            Token::Identifier("table".to_string()),
            Token::OpenBracket,
            Token::Identifier("firstColumn".to_string()),
            Token::Comma,
            Token::Identifier("secondColumn".to_string()),
            Token::Comma,
            Token::Identifier("thirdColumn".to_string()),
            Token::Comma,
            Token::Identifier("fourthColumn".to_string()),
            Token::CloseBracket,
            Token::Values,
            Token::OpenBracket,
            Token::StringLiteral("first".to_string()),
            Token::Comma,
            Token::StringLiteral("second".to_string()),
            Token::Comma,
            Token::StringLiteral("3".to_string()),
            Token::Comma,
            Token::StringLiteral("4th".to_string()),
            Token::CloseBracket,
            Token::Semicolon,
            Token::Eof,
        ];


        let mut generated_tokens = Vec::new();
        loop {
            let token = tokenizer.get_next_token()?;
            let is_eof = token == Token::Eof;
            generated_tokens.push(token);
            if is_eof {
            break;
            }
        }

        assert_eq!(expected_tokens, generated_tokens);
        Ok(())
    }
}
