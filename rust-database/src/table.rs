use std::collections::HashMap;
use crate::constraint_state::{ConstraintState};
use crate::schema::Schema;
use crate::row::{Row, Value, RowErrors}; 
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TableErrors {
    #[error("Row construction failed: {0}")]
    RowConstructionError(#[from] RowErrors),

    #[error("Row with index {0} does not exist")]
    RowNotFound(u64),
}

#[derive(Debug)]
pub struct Table {
    pub schema: Schema,
    pub rows: HashMap<u64, Row>,
    pub constraint_state: ConstraintState,
}

impl Table {
    pub fn new(schema: Schema) -> Self {
        let constraint_state = ConstraintState::new(&schema);
        Table {
            schema,
            rows: HashMap::new(),
            constraint_state,
        }
    }

    pub fn add_row(&mut self, row_values: Vec<Value>) -> Result<u64, TableErrors> {
        let row = Row::new(&self.schema, &mut self.constraint_state, row_values)?; // Validate row
        let row_id = self.rows.len() as u64;
        self.rows.insert(row_id, row);
        Ok(row_id)
    }

    pub fn delete_row(&mut self, index: u64) -> Result<(), TableErrors> {
        if self.rows.remove(&index).is_none() {
            return Err(TableErrors::RowNotFound(index));
        }
        Ok(())
    }

    pub fn edit_row(&mut self, index: u64, row_values: Vec<Value>) -> Result<(), TableErrors> {
        if !self.rows.contains_key(&index) {
            return Err(TableErrors::RowNotFound(index));
        }

        let row = Row::new(&self.schema, &mut self.constraint_state, row_values)?;
        self.rows.insert(index, row);
        Ok(())
    }

    pub fn get_row(&self, index: u64) -> Option<&Row> {
        self.rows.get(&index)
    }
}


#[cfg(test)]
mod table_tests {
    use super::*;
    use crate::column::{Column, DataType};
    use crate::schema::{Schema};

    // ---------- Helpers ----------
    fn make_schema() -> Schema {
        Schema::new(vec![
            Column { name: "id".to_string(), data_type: DataType::Integer, constraints: HashMap::new()},
            Column { name: "name".to_string(), data_type: DataType::String, constraints: HashMap::new()},
        ]).unwrap()
    }

    fn make_table() -> Table {
        Table::new(make_schema())
    }

    fn row_int_str(id: i64, name: &str) -> Vec<Value> {
        vec![Value::Integer(id), Value::String(name.to_string())]
    }

    fn assert_row_eq(table: &Table, key: u64, expected: Vec<Value>) {
        let stored_row = table.rows.get(&key).unwrap();
        assert_eq!(stored_row.values, expected);
    }

    // ---------- Tests ----------
    #[test]
    fn add_row_success() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice")).unwrap();

        assert_eq!(table.rows.len(), 1);
        assert_row_eq(&table, 0, row_int_str(1, "Alice"));
    }

    #[test]
    fn add_row_type_mismatch() {
        let mut table = make_table();

        let bad_row = vec![
            Value::String("Not an integer".to_string()),
            Value::String("Alice".to_string()),
        ];

        let result = table.add_row(bad_row);
        assert!(result.is_err());
    }

    #[test]
    fn delete_row_success() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice")).unwrap();
        table.add_row(row_int_str(2, "Bob")).unwrap();

        assert_eq!(table.rows.len(), 2);

        table.delete_row(0).unwrap();

        assert_eq!(table.rows.len(), 1);
        assert!(table.rows.get(&0).is_none());
        assert!(table.rows.get(&1).is_some());
    }

    #[test]
    fn edit_row_success() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice")).unwrap();

        let new_row = row_int_str(1, "Bob");
        table.edit_row(0, new_row.clone()).unwrap();

        assert_row_eq(&table, 0, new_row);
    }

    #[test]
    fn edit_row_type_mismatch() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice")).unwrap();

        let invalid_row = vec![
            Value::String("Not an integer".to_string()),
            Value::String("Bob".to_string()),
        ];

        let result = table.edit_row(0, invalid_row);
        assert!(result.is_err());
    }
}
