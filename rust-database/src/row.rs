use crate::schema::{Schema};
use crate::column::{Constraint, ConstraintKind};
use std::collections::HashMap;

// ========================================================================================
// ENUMS
// ========================================================================================
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    String(String),
    Integer(i64),
    Null,
}
#[derive(Debug, PartialEq, Eq)]
pub enum RowErrors {
    WrongValueCount { expected: usize, got: usize },
    TypeMismatch {
        column: String,
        expected: crate::column::DataType,
        got: Value,
        got_type: crate::column::DataType,
    },
    NotNullViolated { column: String },
}

impl std::fmt::Display for RowErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RowErrors::WrongValueCount { expected, got } => {
                write!(
                    f,
                    "Row validation failed: expected {} values for schema, but got {}.",
                    expected, got
                )
            }
            RowErrors::TypeMismatch { column, expected, got, got_type } => {
                write!(
                    f,
                    "Type mismatch for column '{}': expected {:?}, but got value {:?} with type {:?}",
                    column, expected, got, got_type
                )
            }
            RowErrors::NotNullViolated { column } => {
                write!(f, "NotNull constraint violated for column '{}'", column)
            }
        }
    }
}

impl std::error::Error for RowErrors {}

// ========================================================================================
// STRUCT
// ========================================================================================
#[derive(Debug, PartialEq, Eq)]
pub struct Row {
    pub values: Vec<Value> 
}

// ========================================================================================
// IMPLEMENTATIONS
// ========================================================================================
impl Row {
    pub fn new(schema: &Schema, mut values: Vec<Value>) -> Result<Self, RowErrors> {
        Self::validate_and_apply_constraints(&mut values, schema)?;
        Ok(Row { values })
    }

    fn validate_and_apply_constraints(values: &mut Vec<Value>, schema: &Schema) -> Result<(), RowErrors> {
        if values.len() != schema.columns.len() {
            return Err(RowErrors::WrongValueCount {
                expected: schema.columns.len(),
                got: values.len(),
            });
        }

        for (i, column) in schema.columns.iter().enumerate() {
            let value_ref = &mut values[i];

            // 1. Type Check: A Null value is permissible at this stage.
            if let Value::Null = value_ref {
            } else if value_ref.get_data_type() != column.data_type {
                return Err(RowErrors::TypeMismatch {
                    column: column.name.clone(),
                    expected: column.data_type,
                    got: value_ref.clone(),
                    got_type: value_ref.get_data_type(),
                });
            }

            // 2. Apply Constraints: Take ownership of the value to process it.
            let current_value = std::mem::replace(value_ref, Value::Null);
            *value_ref =
                match Self::apply_column_constraints(current_value, &column.constraints, &column.name) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
        }
        Ok(())
    }

    fn apply_column_constraints(
        mut value: Value,
        column_constraints: &HashMap<ConstraintKind, Constraint>,
        column_name: &str,
    ) -> Result<Value, RowErrors> {
        // Step 1: Apply the Default constraint if the current value is Null.
        if value == Value::Null {
            if let Some(Constraint::WithValue(_, default_value)) =
                column_constraints.get(&ConstraintKind::Default)
            {
                value = default_value.clone();
            }
        }

        // Step 2: Check the NotNull constraint. This is done *after* the default is applied.
        if column_constraints.contains_key(&ConstraintKind::NotNull) {
            if value == Value::Null {
                // If the value is still Null, it means there was no default,
                // so we violate the NotNull constraint.
                return Err(RowErrors::NotNullViolated {
                    column: column_name.to_string(),
                });
            }
        }

        Ok(value)
    }
}
// TESTS
// ========================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Column, ColumnBuilder};
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

        Row::validate_and_apply_constraints(&mut values, &schema);

        assert_eq!(values, vec![Value::Integer(0)]);
    }

    #[test]
    fn test_not_null_with_provided_value() {
        let column = ColumnBuilder::new("id", DataType::Integer)
            .not_null()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Integer(123)];

        // This should not panic.
        Row::validate_and_apply_constraints(&mut values, &schema);

        assert_eq!(values, vec![Value::Integer(123)]);
    }

    #[test]
    #[should_panic(expected = "NotNull constraint violated for column 'id'")]
    fn test_not_null_with_null_value_panics() {
        let column = ColumnBuilder::new("id", DataType::Integer)
            .not_null()
            .build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Null];

        Row::validate_and_apply_constraints(&mut values, &schema);
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
        Row::validate_and_apply_constraints(&mut values, &schema);

        assert_eq!(values, vec![Value::String("active".to_string())]);
    }

    #[test]
    fn test_nullable_column_accepts_null() {
        // A column without a NotNull constraint is nullable by default.
        let column = ColumnBuilder::new("description", DataType::String).build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Null];

        // This should not panic.
        Row::validate_and_apply_constraints(&mut values, &schema);

        assert_eq!(values, vec![Value::Null]);
    }

    #[test]
    #[should_panic(expected = "Type mismatch for column 'name'")]
    fn test_row_validator_panics_on_type_mismatch() {
        let column = ColumnBuilder::new("name", DataType::String).build();
        let schema = create_test_schema(vec![column]);
        let mut values = vec![Value::Integer(123)]; // Incorrect type.

        Row::validate_and_apply_constraints(&mut values, &schema);
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

        let row = Row::new(
            &schema,
            vec![
                Value::Integer(1),
                Value::String("Alice".to_string()),
                Value::Null, // This should become "guest".
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
}
