// ================================
// database.rs
// Our Database and subsequent tests. We store the Database and the Tables.
// The tables are passed in as we'll have a seperate persistence layer to use.
// ================================
use std::collections::HashMap;
use crate::table::{Table};
use crate::schema::{Schema};

pub struct Database {
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database { 
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: String, schema:Schema) {
        self.tables.insert(name, Table::new(schema));
    }

    pub fn update_table_name(&mut self, name: String, new_name: String) {
        if let Some(table) = self.tables.remove(&name) {
            self.tables.insert(new_name, table);
        }
    }

    pub fn delete_table(&mut self, name: String){
        self.tables.remove(&name);
    }

}

mod table_crud_tests {
    use crate::schema::{ Column, Schema, DataType };

   fn test_schema() -> Schema {
        Schema::new(vec![
            Column { name: "name".into(), data_type: DataType::String, index: 0, constraints: None },
            Column { name: "age".into(),  data_type: DataType::Integer, index: 1, constraints: None },
        ])
    }
    
    #[test]
    fn test_create_table() {
        let mut db = super::Database::new();
        db.create_table("users".to_string(), test_schema());
        assert!(db.tables.contains_key("users"));
    }

    #[test]
    fn test_update_table_name() {
        let mut db = super::Database::new();
        db.create_table("name".to_string(), test_schema());
        db.update_table_name("name".to_string(), "new_name".to_string());
        assert!(db.tables.contains_key("new_name"));
        assert!(!db.tables.contains_key("name"));
    }

    #[test]
    fn test_delete_table_successfully() {
        let mut db = super::Database::new();
        db.create_table("name".to_string(), test_schema());
        db.delete_table("name".to_string());
        assert!(!db.tables.contains_key("name"));
    }
}