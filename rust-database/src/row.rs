use crate::schema::{Schema};
use crate::column::{DataType,Column};
use crate::constraint_state::{ConstraintState};
use thiserror::Error;


// ========================================================================================
// ENUMS
// ========================================================================================
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Value {
    String(String),
    Integer(i64),
    Null,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum RowErrors {
    #[error("Row validation failed: expected {expected} values for schema, but got {got}.")]
    WrongValueCount { expected: usize, got: usize },

    #[error("Type mismatch for column '{column}': expected {expected:?}, but got value {got:?} with type {got_type:?}")]
    TypeMismatch {
        column: String,
        expected: crate::column::DataType,
        got: Value,
        got_type: crate::column::DataType,
    },

    #[error("NotNull constraint violated for column '{column}'")]
    NotNullViolated { column: String },

    #[error("Unique constraint violated for column '{column}' with value {value:?}")]
    UniqueViolated { column: String, value: Value },
}

// ========================================================================================
// STRUCT
// ========================================================================================
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub values: Vec<Value> 
}

// ========================================================================================
// IMPLEMENTATIONS
// ========================================================================================
impl Row {
    pub fn new(
        schema: &Schema,
        constraint_state: &mut ConstraintState,
        mut values: Vec<Value>,
    ) -> Result<Self, RowErrors> {
        Self::validate_value_count(&values, schema)?;
        Self::validate_and_apply_constraints(&mut values, schema, constraint_state)?;
        Ok(Row { values })
    }

    fn validate_value_count(values: &[Value], schema: &Schema) -> Result<(), RowErrors> {
        if values.len() != schema.columns.len() {
            return Err(RowErrors::WrongValueCount {
                expected: schema.columns.len(),
                got: values.len(),
            });
        }
        Ok(())
    }

    fn validate_and_apply_constraints(
        values: &mut Vec<Value>,
        schema: &Schema,
        constraint_state: &mut ConstraintState,
    ) -> Result<(), RowErrors> {
        for (col, val) in schema.columns.iter().zip(values.iter_mut()) {
            Self::validate_type(val, &col.data_type, &col.name)?;
            Self::apply_default_if_null(val, col, constraint_state);
            Self::check_not_null(val, col, constraint_state)?;
            Self::check_unique(val, col, constraint_state)?;
            Self::check_if_indexed(val, col, constraint_state)?;
        }
        Ok(())
    }

    fn validate_type(val: &Value, expected_type: &DataType, col_name: &str) -> Result<(), RowErrors> {
        if let Value::Null = val {
            Ok(())
        } else if val.get_data_type() != *expected_type {
            Err(RowErrors::TypeMismatch {
                column: col_name.to_string(),
                expected: expected_type.clone(),
                got: val.clone(),
                got_type: val.get_data_type(),
            })
        } else {
            Ok(())
        }
    }

    fn apply_default_if_null(val: &mut Value, col: &Column, constraint_state: &ConstraintState) {
        if *val == Value::Null {
            if let Some(default_val) = constraint_state.default_values.get(&col.name) {
                *val = default_val.clone();
            }
        }
    }

    fn check_not_null(val: &Value, col: &Column, constraint_state: &ConstraintState) -> Result<(), RowErrors> {
        if constraint_state.not_null_columns.contains(&col.name) && *val == Value::Null {
            Err(RowErrors::NotNullViolated {
                column: col.name.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn check_unique(
        val: &Value,
        col: &Column,
        constraint_state: &mut ConstraintState,
    ) -> Result<(), RowErrors> {
        if *val != Value::Null {
            if let Some(seen) = constraint_state.unique_values.get_mut(&col.name) {
                if !seen.insert(val.clone()) {
                    return Err(RowErrors::UniqueViolated {
                        column: col.name.clone(),
                        value: val.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn check_if_indexed(
        val: &Value,
        col: &Column,
        constraint_state: &mut ConstraintState,
    ) -> Result<(), RowErrors> {
        if let Some(index) = constraint_state.indexes.get_mut(&col.name) {
            index.insert(val.clone());
        }
        Ok(())
    }
}



// ========================================================================================
// TESTS
// ========================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Column, ColumnBuilder};
    use crate::constraint_state::ConstraintState;
    use crate::schema::Schema;
    use crate::column::{DataType};


    // Helper to create a simple schema for testing purposes.
    fn create_test_schema(columns: Vec<Column>) -> Schema {
        return Schema::new(columns).unwrap();
    }

    #[test]
    fn test_apply_default_value_on_null() {
        let column = ColumnBuilder::new("status", DataType::Integer)
            .default(Value::Integer(0))
            .unwrap()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Null];
        let mut constraint_state: ConstraintState = ConstraintState::new(&schema);
        Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state).unwrap();

        assert_eq!(values, vec![Value::Integer(0)]);
    }
    
    #[test]
    fn test_unique_constraint_works_correctly(){
        let column = ColumnBuilder::new("Status", DataType::String).unique().build();
        let schema = create_test_schema(vec![column]);

        let mut values = vec![Value::String("unique1".to_string())];
        let mut constraint_state: ConstraintState = ConstraintState::new(&schema);

        // Now make a row, and then the same row again. 

        Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state).unwrap();

        let result = Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state);
        assert!(matches!(
            result,
            Err(RowErrors::UniqueViolated { column, value }) if column == "Status" && value == Value::String("unique1".to_string())
        ));
    }

    #[test]
    fn test_not_null_with_provided_value() {
        let column = ColumnBuilder::new("id", DataType::Integer)
            .not_null()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Integer(123)];

        // This should not panic.
        let mut constraint_state: ConstraintState = ConstraintState::new(&schema);

        // This should not panic.
        Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state).unwrap();
        
        assert_eq!(values, vec![Value::Integer(123)]);
    }

    #[test]
    fn test_not_null_with_null_value_returns_error() {
        let column = ColumnBuilder::new("id", DataType::Integer)
            .not_null()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Null];

        let mut constraint_state = ConstraintState::new(&schema);

        let result = Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state);

        assert!(matches!(
            result,
            Err(RowErrors::NotNullViolated { column }) if column == "id"
        ));
    }

    #[test]
    fn test_not_null_is_satisfied_by_default_value() {
        let column = ColumnBuilder::new("status", DataType::String)
            .not_null()
            .default(Value::String("active".to_string()))
            .unwrap()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Null];

        // This should not panic because the default value is applied first.

        let mut constraint_state = ConstraintState::new(&schema);

        // This should not panic.
        Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state).unwrap();


        assert_eq!(values, vec![Value::String("active".to_string())]);
    }


    #[test]
    fn test_nullable_column_accepts_null() {
        let column = ColumnBuilder::new("description", DataType::String).build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Null];

        let mut constraint_state = ConstraintState::new(&schema);

        // This should not panic.
        Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state).unwrap();

        assert_eq!(values, vec![Value::Null]);
    }

    #[test]
    fn test_row_validator_returns_error_on_type_mismatch() {
        let column = ColumnBuilder::new("name", DataType::String).build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Integer(123)]; // Incorrect type.

        let mut constraint_state = ConstraintState::new(&schema);

        let result = Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state);

        assert!(matches!(
            result,
            Err(RowErrors::TypeMismatch { column, .. }) if column == "name"
        ));
    }

    #[test]
    fn test_full_row_creation_with_constraints() {
        let schema = create_test_schema(vec![
            ColumnBuilder::new("id", DataType::Integer)
                .not_null()
                .build(),
            ColumnBuilder::new("name", DataType::String).build(), // Nullable
            ColumnBuilder::new("role", DataType::String)
                .default(Value::String("guest".to_string()))
                .unwrap()
                .build(),
        ]);

        let mut constraint_state = ConstraintState::new(&schema);
        
        let row = Row::new(
            &schema,
            &mut constraint_state,
            vec![
                Value::Integer(1),
                Value::String("Alice".to_string()),
                Value::Null,
            ],
        ).unwrap();

        assert_eq!(
            row,
            Row {
                values: vec![
                    Value::Integer(1),
                    Value::String("Alice".to_string()),
                    Value::String("guest".to_string()),
                ]
            }
        );
    }

    #[test]
    fn test_new_row_fails_with_wrong_value_count() {
        let schema = create_test_schema(vec![
            ColumnBuilder::new("id", DataType::Integer).build(),
            ColumnBuilder::new("name", DataType::String).build(),
        ]);
        let mut constraint_state = ConstraintState::new(&schema);

        // Too few values
        let result_too_few = Row::new(
            &schema,
            &mut constraint_state,
            vec![Value::Integer(1)],
        );
        assert!(matches!(
            result_too_few,
            Err(RowErrors::WrongValueCount { expected: 2, got: 1 })
        ));

        // Too many values
        let result_too_many = Row::new(
            &schema,
            &mut constraint_state,
            vec![
                Value::Integer(1),
                Value::String("test".to_string()),
                Value::Null,
            ],
        );
        assert!(matches!(
            result_too_many,
            Err(RowErrors::WrongValueCount { expected: 2, got: 3 })
        ));
    }

    #[test]
    fn test_unique_constraint_allows_multiple_nulls() {
        let column = ColumnBuilder::new("email", DataType::String).unique().build();
        let schema = create_test_schema(vec![column]);
        let mut constraint_state = ConstraintState::new(&schema);

        // Insert first row with NULL email
        let result1 = Row::new(&schema, &mut constraint_state, vec![Value::Null]);
        assert!(result1.is_ok());

        // Insert second row with NULL email
        let result2 = Row::new(&schema, &mut constraint_state, vec![Value::Null]);
        
        // This should succeed because NULLs are not compared for uniqueness
        assert!(result2.is_ok());
    }

    #[test]
    fn test_default_value_is_not_applied_on_provided_value() {
        let column = ColumnBuilder::new("status", DataType::Integer)
            .default(Value::Integer(0))
            .unwrap()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Integer(100)]; // A specific, non-default value
        let mut constraint_state = ConstraintState::new(&schema);
        
        Row::validate_and_apply_constraints(&mut values, &schema, &mut constraint_state).unwrap();

        // The value should remain what we provided, not the default
        assert_eq!(values, vec![Value::Integer(100)]);
    }


    #[test]
    fn test_indexed_column_populates_constraint_state() {
        // Assuming ColumnBuilder has an `.indexed()` method
        let column = ColumnBuilder::new("user_id", DataType::Integer).index().build();
        let schema = create_test_schema(vec![column]);
        let mut constraint_state = ConstraintState::new(&schema);

        // Initially, the index for the value should not exist
        let value_to_insert = Value::Integer(12345);
        let index = constraint_state.indexes.get("user_id").unwrap();
        assert!(!index.contains(&value_to_insert));

        // Create a new row, which should trigger `check_if_indexed`
        Row::new(
            &schema,
            &mut constraint_state,
            vec![value_to_insert.clone()],
        ).unwrap();

        // Now, the value should be present in the index
        let index_after = constraint_state.indexes.get("user_id").unwrap();
        assert!(index_after.contains(&value_to_insert));
    }
}
