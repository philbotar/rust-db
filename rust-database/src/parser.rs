
use crate::column::{DataType};
use crate::tokenizer::{Token};
use thiserror::Error;

// ========================================================================================
// ENUM
// ========================================================================================
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected Token '{0}' at position '{1}'")]
    UnexpectedToken(String, usize),

    #[error("Invalid Integer '{0}' at position '{1}'")]
    InvalidInteger(String, usize),
}

#[derive(Debug, PartialEq)]
pub enum Statements {
    Select(SelectStatement),
    Insert(InsertStatement),
    CreateTable(CreateTableStatement),
}

#[derive(Debug, PartialEq)]
pub enum SelectColumn {
    Wildcard,
    Identifier(String),
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    And,
    Or,
}

// The main Expression enum
#[derive(Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    // We use Box to handle recursive data structures, preventing infinite size.
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
}

// ========================================================================================
// STRUCT
// ========================================================================================
#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub from_table: String,
    pub where_clause: Option<Expression>
}

#[derive(Debug, PartialEq)]
pub struct InsertStatement {
    pub table_name: String,
    pub values: Vec<Literal>,
    pub columns: Vec<String>,
}

// Special Case for Create Tables
// =============================================
#[derive(Debug, PartialEq)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    // Potentially add constraints like PRIMARY KEY, NOT NULL later
}

#[derive(Debug, PartialEq)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
}
// =============================================

pub struct Parser { 
    tokens: Vec<Token>, 
    position: usize, // Track which token 
}

// ==============================================================================
// IMPLEMENTATION
// ==============================================================================

impl Parser { 
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse_statement(&mut self) -> Result<Statements, ParserError> {
        let current_token = self.current_token()?.clone();

        match current_token {
            Token::Select => {
                let select_stmt = self.parse_select_statement()?;
                Ok(Statements::Select(select_stmt))
            }
            Token::Insert => {
                Err(ParserError::UnexpectedToken("INSERT".to_string(), self.position))
            },
            Token::CreateTable => {
                Err(ParserError::UnexpectedToken("CREATE TABLE".to_string(), self.position))
            },
            _ => {
                Err(ParserError::UnexpectedToken(format!("{:?}", current_token), self.position))
            }
        }        
    }

    pub fn parse_select_statement(&mut self) -> Result<SelectStatement, ParserError> {
        self.consume_token()?; // Consume SELECT token

        let columns = self.parse_select_columns()?;

        // Expect FROM
        self.expect_token(&Token::From)?;

        // Expect table name
        let from_table = match self.consume_token()? {
            Token::Identifier(name) => name,
            t => {
                return Err(ParserError::UnexpectedToken(
                    format!("Expected table name, found {:?}", t),
                    self.position - 1,
                ))
            }
        };

        let mut where_clause = None;
        if let Ok(Token::Where) = self.current_token() {
            self.consume_token()?; 
            where_clause = Some(self.parse_expression()?);
        }

        self.expect_token(&Token::Semicolon)?;

        Ok(SelectStatement {
            columns,
            from_table,
            where_clause,
        })
    }
    
    /// Parses the column part of a SELECT statement 
    fn parse_select_columns(&mut self) -> Result<Vec<SelectColumn>, ParserError> {
        let mut columns = vec![];

        // Handle wildcard *
        if let Ok(Token::Asterisk) = self.current_token() {
            self.consume_token()?;
            columns.push(SelectColumn::Wildcard);
            return Ok(columns);
        }

        // Handle one or more comma-separated identifiers
        loop {
            match self.consume_token()? {
                Token::Identifier(name) => columns.push(SelectColumn::Identifier(name)),
                t => {
                    return Err(ParserError::UnexpectedToken(
                        format!("Expected column name or '*', found {:?}", t),
                        self.position - 1,
                    ))
                }
            }
            // If the next token is not a comma, we're done with columns
            if let Ok(Token::Comma) = self.current_token() {
                self.consume_token()?;
            } else {
                break;
            }
        }
        Ok(columns)
    }


    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let left = match self.consume_token()? {
            Token::Identifier(name) => Expression::Identifier(name),
            t => {
                return Err(ParserError::UnexpectedToken(
                    format!("Expected identifier in expression, found {:?}", t),
                    self.position - 1,
                ))
            }
        };

        // Operator
        let op = self.match_binary_operator()?;

        // Right-hand side (expecting a literal)
        let right = match self.consume_token()? {
            Token::StringLiteral(s) => Expression::Literal(Literal::String(s)),
            Token::NumericLiteral(n) => {
                let val = n.parse::<i64>().map_err(|_| {
                    ParserError::InvalidInteger(n.clone(), self.position - 1)
                })?;
                Expression::Literal(Literal::Integer(val))
            }
            t => {
                return Err(ParserError::UnexpectedToken(
                    format!("Expected literal in expression, found {:?}", t),
                    self.position - 1,
                ))
            }
        };

        Ok(Expression::Binary(Box::new(left), op, Box::new(right)))
    }


    // ==============================================================================
    // UTILITY FUNCTIONS
    // ==============================================================================

    /// Consumes the current token only if it matches the expected one
    fn expect_token(&mut self, expected: &Token) -> Result<Token, ParserError> {
        let token = self.consume_token()?;
        if &token == expected {
            Ok(token)
        } else {
            Err(ParserError::UnexpectedToken(
                format!("Expected {:?}, found {:?}", expected, token),
                self.position - 1,
            ))
        }
    }

    pub fn current_token(&self) -> Result<&Token, ParserError> {
        if self.position < self.tokens.len() {
            Ok(&self.tokens[self.position])
        } else {
            Err(ParserError::UnexpectedToken("End of input".to_string(), self.position))
        }
    }

    pub fn consume_token(&mut self) -> Result<Token, ParserError> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone(); // Clone to return by value
            self.position += 1;
            Ok(token)
        } else {
            Err(ParserError::UnexpectedToken("End of input".to_string(), self.position))
        }
    }

    fn match_binary_operator(&mut self) -> Result<BinaryOperator, ParserError> {
        let token = self.consume_token()?; // Consume the operator token
        match token {
            Token::Equals => Ok(BinaryOperator::Equals),
            Token::NotEquals => Ok(BinaryOperator::NotEquals),
            Token::GreaterThan => Ok(BinaryOperator::GreaterThan),
            Token::LessThan => Ok(BinaryOperator::LessThan),
            Token::GreaterThanOrEquals => Ok(BinaryOperator::GreaterThanOrEquals),
            Token::LessThanOrEquals => Ok(BinaryOperator::LessThanOrEquals),
            Token::And => Ok(BinaryOperator::And),
            Token::Or => Ok(BinaryOperator::Or),
            t => Err(ParserError::UnexpectedToken(
                format!("Expected binary operator, found {:?}", t),
                self.position - 1,
            )),
        }
    }


}

// ==============================================================================
// TESTS
// ==============================================================================
// The Parser will be taking in a Vec of tokens.
// We need to just pass them in and expect an AST out. 
#[cfg(test)]
mod tests { 
    use super::*;
    use crate::tokenizer::{Token};

    #[test]
    fn test_with_select() {
        let tokens = vec![
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

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().unwrap();

        // Compare the AST to the Tokens passed in. 
        // We are expecting a Select Statement
        let expected_statement = Statements::Select(SelectStatement {
            columns: vec![SelectColumn::Wildcard],
            from_table: "table".to_string(),
            where_clause: Some(Expression::Binary(
                Box::new(Expression::Identifier("name".to_string())),
                BinaryOperator::Equals,
                Box::new(Expression::Literal(Literal::String("PHILIP".to_string()))),
            )),
        });

        assert_eq!(statement, expected_statement);

    }

    #[test]
    fn test_select_specific_columns_no_where() {
        let tokens = vec![
            Token::Select,
            Token::Identifier("col1".to_string()),
            Token::Comma,
            Token::Identifier("col2".to_string()),
            Token::From,
            Token::Identifier("my_table".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement().unwrap();

        let expected_statement = Statements::Select(SelectStatement {
            columns: vec![
                SelectColumn::Identifier("col1".to_string()),
                SelectColumn::Identifier("col2".to_string()),
            ],
            from_table: "my_table".to_string(),
            where_clause: None,
        });

        assert_eq!(statement, expected_statement);
    }

    #[test]
    fn test_select_with_numeric_where_clause() {
        let tokens = vec![
            Token::Select,
            Token::Asterisk,
            Token::From,
            Token::Identifier("users".to_string()),
            Token::Where,
            Token::Identifier("id".to_string()),
            Token::Equals,
            Token::NumericLiteral("123".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement().unwrap();

        let expected_statement = Statements::Select(SelectStatement {
            columns: vec![SelectColumn::Wildcard],
            from_table: "users".to_string(),
            where_clause: Some(Expression::Binary(
                Box::new(Expression::Identifier("id".to_string())),
                BinaryOperator::Equals,
                Box::new(Expression::Literal(Literal::Integer(123))),
            )),
        });

        assert_eq!(statement, expected_statement);
    }

    #[test]
    fn test_insert_statement_error() {
        let tokens = vec![
            Token::Insert,
            Token::Identifier("INTO".to_string()),
            Token::Identifier("my_table".to_string()),
            Token::Identifier("VALUES".to_string()),
            Token::OpenBracket,
            Token::StringLiteral("value1".to_string()),
            Token::Comma,
            Token::NumericLiteral("123".to_string()),
            Token::CloseBracket,
            Token::Semicolon,
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let error = parser.parse_statement().unwrap_err();

        assert_eq!(
            error.to_string(),
            "Unexpected Token 'INSERT' at position '0'"
        );
    }

    #[test]
    fn test_create_table_statement_error() {
        let tokens = vec![
            Token::CreateTable,
            Token::Identifier("new_table".to_string()),
            Token::OpenBracket,
            Token::Identifier("id".to_string()),
            Token::Identifier("INTEGER".to_string()),
            Token::Comma,
            Token::Identifier("name".to_string()),
            Token::Identifier("STRING".to_string()),
            Token::CloseBracket,
            Token::Semicolon,
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let error = parser.parse_statement().unwrap_err();

        assert_eq!(
            error.to_string(),
            "Unexpected Token 'CREATE TABLE' at position '0'"
        );
    }

}
