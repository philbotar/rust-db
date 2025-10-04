use crate::column::{Column, DataType};
use crate::row::Value;
use std::collections::HashMap;
use thiserror::Error;


// ========================================================================================
// ENUMS
// ========================================================================================
#[derive(Debug, PartialEq, Error)]
pub enum SchemaError {
    #[error("Duplicate column name: {0}")]
    DuplicateColumnName(String),
    #[error("Index out of bounds for column '{name}': index {index}, but length is {len}")]
    IndexOutOfBounds { name: String, index: usize, len: usize },
    #[error("Default value type mismatch for column '{column_name}'")]
    DefaultValueTypeMismatch { column_name: String },
}

// ========================================================================================
// STRUCTS
// ========================================================================================
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub columns: Vec<Column>,
    pub name_to_index: HashMap<String, usize>, // fast lookup
}


// ========================================================================================
// IMPLEMENTATIONS
// ========================================================================================
impl Schema {
    pub fn new(columns: Vec<Column>) -> Result<Self, SchemaError> {
        Self::validate_default_value_types(&columns)?;
        let name_to_index = Self::build_name_to_index_map(&columns)?;
        Ok(Self { columns, name_to_index })
    }

    fn validate_default_value_types(columns: &[Column]) -> Result<(), SchemaError> {
        for col in columns.iter() {
            if let Some(constraint) = col.constraints.get(&crate::constraint_state::ConstraintKind::Default) {
                if let crate::constraint_state::Constraint::WithValue(_, val) = constraint {
                    if val.get_data_type() != col.data_type && val.get_data_type() != DataType::Null {
                        return Err(SchemaError::DefaultValueTypeMismatch { column_name: col.name.clone() });
                    }
                }
            }
        }
        Ok(())
    }

    fn build_name_to_index_map(columns: &[Column]) -> Result<HashMap<String, usize>, SchemaError> {
        let mut name_to_index: HashMap<String, usize> = HashMap::with_capacity(columns.len());

        for (i, col) in columns.iter().enumerate() {
            if name_to_index.insert(col.name.clone(), i).is_some() {
                return Err(SchemaError::DuplicateColumnName(col.name.clone()));
            }
        }

        Ok(name_to_index)
    }

    pub fn get_column_by_name(&self, name: &str) -> Option<&Column> {
        self.name_to_index.get(name).map(|&idx| &self.columns[idx])
    }
    
    pub fn get_column_by_index(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }

// get_column_index(&self, name: &str) -> Option<usize>
// column_count(&self) -> usize

}


impl Value {
    pub fn get_data_type(&self) -> DataType {
        match self {
            Value::String(_) => DataType::String,
            Value::Integer(_) => DataType::Integer,
            Value::Null => DataType::Null,
        }
   }
}

// ========================================================================================
// TESTS
// ========================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{Column, ColumnBuilder, DataType};

    struct SchemaBuilder {
        columns: Vec<Column>,
    }

    impl SchemaBuilder {
        fn new() -> Self {
            Self {
                columns: Vec::new(),
            }
        }

        fn add_column(mut self, column: Column) -> Self {
            self.columns.push(column);
            self
        }

        fn build(self) -> Result<Schema, SchemaError> {
            Schema::new(self.columns)
        }
    }

    #[test]
    fn test_duplicate_name() {
        let result = SchemaBuilder::new()
            .add_column(ColumnBuilder::new("id", DataType::Integer).build())
            .add_column(ColumnBuilder::new("id", DataType::String).build())
            .build();

        assert_eq!(result, Err(SchemaError::DuplicateColumnName("id".to_string())));
    }

    #[test]
    fn test_valid_schema() {
        let result = SchemaBuilder::new()
            .add_column(ColumnBuilder::new("id", DataType::Integer).build())
            .add_column(ColumnBuilder::new("name", DataType::String).build())
            .build();

        assert!(result.is_ok());
    }
}
