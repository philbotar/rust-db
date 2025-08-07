use std::collections::HashMap;
use crate::schema::{Schema};
use crate::row::{Row, Value};

#[derive(Debug)]
pub struct Table {
    pub schema: Schema,
    pub rows: HashMap<u64, Row>,
}

impl Table {
    pub fn new(schema: Schema) -> Self {
        Table {
            schema,
            rows: HashMap::new(),
        }
    }

    pub fn add_row(&mut self, row_values: Vec<Value>) {
        let row = Row::new(&self.schema, row_values);
        let row_id = self.rows.len() as u64;
        self.rows.insert(row_id, Row { values: row.values });
    }

    pub fn delete_row(&mut self, index: u64){
        self.rows.remove(&index);
    }

    pub fn edit_row(&mut self, index: u64, row_values: Vec<Value>) {
        let row = Row::new(&self.schema, row_values);
        self.rows.insert(index, Row { values: row.values });
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

        table.add_row(row_int_str(1, "Alice"));

        assert_eq!(table.rows.len(), 1);
        assert_row_eq(&table, 0, row_int_str(1, "Alice"));
    }

    #[test]
    #[should_panic(expected = "Type mismatch")]
    fn add_row_type_mismatch() {
        let mut table = make_table();

        let bad_row = vec![
            Value::String("Not an integer".to_string()),
            Value::String("Alice".to_string()),
        ];

        table.add_row(bad_row);
    }

    #[test]
    fn delete_row_success() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice"));
        table.add_row(row_int_str(2, "Bob"));

        assert_eq!(table.rows.len(), 2);

        table.delete_row(0);

        assert_eq!(table.rows.len(), 1);
        assert!(table.rows.get(&0).is_none());
        assert!(table.rows.get(&1).is_some());
    }

    #[test]
    fn edit_row_success() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice"));

        let new_row = row_int_str(1, "Bob");
        table.edit_row(0, new_row.clone());

        assert_row_eq(&table, 0, new_row);
    }

    #[test]
    #[should_panic(expected = "Type mismatch")]
    fn edit_row_type_mismatch() {
        let mut table = make_table();

        table.add_row(row_int_str(1, "Alice"));

        let invalid_row = vec![
            Value::String("Not an integer".to_string()),
            Value::String("Bob".to_string()),
        ];

        table.edit_row(0, invalid_row);
    }
}
