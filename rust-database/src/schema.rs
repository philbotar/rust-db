use crate::column::{Column, DataType};
use crate::row::Value;
use std::collections::{HashMap, HashSet};


// ========================================================================================
// ENUMS
// ========================================================================================
#[derive(Debug, PartialEq)]
pub enum SchemaError {
    DuplicateColumnName(String),
    IndexOutOfBounds { name: String, index: usize, len: usize },
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

// Notes: the schema should only ever exist if all the columns are correct. 
// We do NOT create a schema otherwise. We pass in the columns at once. 
impl Schema {
    pub fn new(columns: Vec<Column>) -> Result<Self, SchemaError> {
        // ? Postpend means it returns early on failure. 
        Self::validate_column_uniqueness(&columns)?;
        let name_to_index = Self::create_name_to_index(&columns)?;
        Ok(Self { columns, name_to_index })
    }

    // Validates that theres uniqueness when there needs to be. We are borrowing columns, not taking the real values. 
    fn validate_column_uniqueness(columns: &[Column]) -> Result<(), SchemaError> {
        let mut column_names: HashSet<&str> = HashSet::new();

        for col in columns.iter() {
            if !column_names.insert(&col.name) {
                return Err(SchemaError::DuplicateColumnName(col.name.clone()));
            }
        }

        Ok(())
    }

    fn create_name_to_index(columns: &[Column]) -> Result<HashMap<String, usize>, SchemaError> {
        let mut name_to_index: HashMap<String, usize> = HashMap::new();

        for (i, col) in columns.iter().enumerate() {

            if name_to_index.contains_key(&col.name) {
                return Err(SchemaError::DuplicateColumnName(col.name.clone()));
            }

            name_to_index.insert(col.name.clone(), i);
        }
        
        Ok(name_to_index)
    }
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
