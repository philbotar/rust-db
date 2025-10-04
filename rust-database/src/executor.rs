// executor.rs

use crate::parser::{Statements, SelectStatement, Expression, BinaryOperator, SelectColumn, Literal};
use crate::row::{Row, Value};
use crate::schema::{Schema};
use crate::database::{Database};

#[derive(Debug, PartialEq)] // Added for testing
pub struct QueryResult {
    pub rows: Vec<Row>,
}

#[derive(Debug)]
pub enum ExecutionError {
    TableNotFound,
    ColumnNotFound(String),
    InvalidExpression,
    TypeMismatch,
}


// ==============================================================================
// EXECUTOR IMPLEMENTATION
// ==============================================================================
pub struct Executor {}

impl Executor {
    pub fn execute(&self, ast: &Statements, db: &Database) -> Result<QueryResult, ExecutionError> {
        match ast {
            Statements::Select(stmt) => self.execute_select(stmt, db),
            _ => unimplemented!(),
        }
    }

    fn execute_select(
        &self,
        stmt: &SelectStatement,
        db: &Database,
    ) -> Result<QueryResult, ExecutionError> {
        let table = db.get_table(stmt.from_table.clone()).map_err(|_| ExecutionError::TableNotFound)?;

        let filtered_rows: Vec<Row> = table
            .rows
            .values()
            .filter_map(|row| {
                let should_include = match &stmt.where_clause {
                    Some(expression) => self.evaluate_expression(expression, row, &table.schema).ok(),
                    None => Some(true),
                };

                if should_include.unwrap_or(false) {
                    Some(row.clone())
                } else {
                    None
                }
            })
            .collect();

        let final_rows = self.project_columns(&filtered_rows, &stmt.columns, &table.schema)?;
        Ok(QueryResult { rows: final_rows })
    }

    fn evaluate_expression(
        &self,
        expr: &Expression,
        row: &Row,
        schema: &Schema,
    ) -> Result<bool, ExecutionError> {
        match expr {
            Expression::Binary(left, op, right) => {
                let left_val = self.resolve_value(left, row, schema)?;
                let right_val = self.resolve_value_from_literal(right)?; 

                match op {
                    BinaryOperator::Equals => Ok(left_val == &right_val),
                    _ => unimplemented!("Operator not supported yet"),
                }
            }
            _ => unimplemented!("Expression type not supported yet"),
        }
    }

    fn resolve_value<'a>(
        &self,
        expr: &'a Expression,
        row: &'a Row,
        schema: &Schema,
    ) -> Result<&'a Value, ExecutionError> {
        match expr {
            Expression::Identifier(col_name) => {
                let col_index = schema.get_column_index(col_name)
                    .ok_or_else(|| ExecutionError::ColumnNotFound(col_name.clone()))?;
                Ok(&row.values[col_index])
            }
            _ => Err(ExecutionError::InvalidExpression),
        }
    }
    
    // Helper specifically for the right-hand side of a simple binary expression
    fn resolve_value_from_literal(&self, expr: &Expression) -> Result<Value, ExecutionError> {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Integer(i) => Ok(Value::Integer(*i)),
                Literal::String(s) => Ok(Value::String(s.clone())),
                _ => unimplemented!(),
            },
            _ => Err(ExecutionError::InvalidExpression),
        }
    }
    
    fn project_columns(&self, rows: &[Row], columns: &[SelectColumn], schema: &Schema) -> Result<Vec<Row>, ExecutionError> {
        if columns.len() == 1 && columns[0] == SelectColumn::Wildcard {
            return Ok(rows.to_vec()); // Return all columns
        }

        let mut projected_rows = Vec::new();
        let mut col_indices = Vec::new();

        for col in columns {
            if let SelectColumn::Identifier(name) = col {
                let index = schema.get_column_index(name)
                    .ok_or_else(|| ExecutionError::ColumnNotFound(name.clone()))?;
                col_indices.push(index);
            }
        }

        for row in rows {
            let projected_values = col_indices.iter().map(|&i| row.values[i].clone()).collect();
            projected_rows.push(Row { values: projected_values });
        }
        
        Ok(projected_rows)
    }
}


// ==============================================================================
// TESTS
// ==============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{ColumnBuilder, DataType};
    use crate::database::Database;
    use crate::row::{Value};
    use crate::schema::Schema;

    // ===== Test Setup =====
    fn create_mock_db() -> Database {
        let schema = Schema::new(vec![
            ColumnBuilder::new("id", DataType::Integer).build(),
            ColumnBuilder::new("name", DataType::String).build(),
            ColumnBuilder::new("age", DataType::Integer).build(),
        ])
        .unwrap();

        let mut db = Database::new();
        db.create_table("users".to_string(), schema).unwrap();
        let table = db.get_table_mut("users".to_string()).unwrap();

        table
            .add_row(vec![
                Value::Integer(1),
                Value::String("Alice".to_string()),
                Value::Integer(30),
            ])
            .unwrap();
        table
            .add_row(vec![
                Value::Integer(2),
                Value::String("Bob".to_string()),
                Value::Integer(25),
            ])
            .unwrap();
        table
            .add_row(vec![
                Value::Integer(3),
                Value::String("Charlie".to_string()),
                Value::Integer(30),
            ])
            .unwrap();
        db
    }
    
    #[test]
    fn test_select_all_no_where() {
        let db = create_mock_db();
        let executor = Executor {};
        let ast = Statements::Select(SelectStatement {
            from_table: "users".to_string(),
            columns: vec![SelectColumn::Wildcard],
            where_clause: None,
        });

        let result = executor.execute(&ast, &db).unwrap();
        // Should return all 3 rows
        assert_eq!(result.rows.len(), 3);
    }
    
    #[test]
    fn test_select_with_integer_where_clause() {
        let db = create_mock_db();
        let executor = Executor {};
        let ast = Statements::Select(SelectStatement {
            from_table: "users".to_string(),
            columns: vec![SelectColumn::Wildcard],
            where_clause: Some(Expression::Binary(
                Box::new(Expression::Identifier("id".to_string())),
                BinaryOperator::Equals,
                Box::new(Expression::Literal(Literal::Integer(2))),
            )),
        });
        
        let result = executor.execute(&ast, &db).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].values[1], Value::String("Bob".to_string()));
    }
    
    #[test]
    fn test_select_with_string_where_clause() {
        let db = create_mock_db();
        let executor = Executor {};
        let ast = Statements::Select(SelectStatement {
            from_table: "users".to_string(),
            columns: vec![SelectColumn::Wildcard],
            where_clause: Some(Expression::Binary(
                Box::new(Expression::Identifier("name".to_string())),
                BinaryOperator::Equals,
                Box::new(Expression::Literal(Literal::String("Charlie".to_string()))),
            )),
        });
        
        let result = executor.execute(&ast, &db).unwrap();
        
        // Should return only Charlie's row
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].values[0], Value::Integer(3));
    }
    
    #[test]
    fn test_select_with_projection() { // Projection is selecting a subset of rows.
        let db = create_mock_db();
        let executor = Executor {};

        let ast = Statements::Select(SelectStatement {
            from_table: "users".to_string(),
            columns: vec![
                SelectColumn::Identifier("name".to_string()),
                SelectColumn::Identifier("age".to_string()),
            ],
            where_clause: Some(Expression::Binary(
                Box::new(Expression::Identifier("age".to_string())),
                BinaryOperator::Equals,
                Box::new(Expression::Literal(Literal::Integer(30))),
            )),
        });

        let result = executor.execute(&ast, &db).unwrap();
        
        // Should return 2 rows (Alice and Charlie)
        assert_eq!(result.rows.len(), 2);
        // Each row should only have 2 columns (name, age)
        assert_eq!(result.rows[0].values.len(), 2);
        assert_eq!(result.rows[1].values.len(), 2);
        // Check the values
        assert_eq!(result.rows[0].values, vec![Value::String("Alice".to_string()), Value::Integer(30)]);
        assert_eq!(result.rows[1].values, vec![Value::String("Charlie".to_string()), Value::Integer(30)]);
    }
}
