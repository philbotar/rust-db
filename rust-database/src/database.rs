// ================================
// database.rs
// Our Database and subsequent tests. We store the Database and the Tables.
// The tables are passed in as we'll have a seperate persistence layer to use.
// ================================
use std::collections::HashMap;
use crate::table::{Table};
use crate::schema::{Schema};


// ========================================================================================
// ERRORS
// ========================================================================================

#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    DuplicateTableName(String),
    TableNotFound { name: String },
}

// ========================================================================================
// STRUCTS
// ========================================================================================
pub struct Database {
    tables: HashMap<String, Table>,
}

// ========================================================================================
// IMPLEMENTATION
// ========================================================================================
impl Database {
    pub fn new() -> Self {
        Database { 
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: String, schema: Schema) -> Result<(), DatabaseError> {
        if self.tables.contains_key(&name) {
            return Err(DatabaseError::DuplicateTableName(name));
        }

        self.tables.insert(name, Table::new(schema));
        Ok(())
    }

    pub fn update_table_name(&mut self, name: String, new_name: String) -> Result<(), DatabaseError> {
        if !self.tables.contains_key(&name) {
            return Err(DatabaseError::TableNotFound { name });
        }

        if self.tables.contains_key(&new_name) {
            return Err(DatabaseError::DuplicateTableName(new_name));
        }

        if let Some(table) = self.tables.remove(&name) {
            self.tables.insert(new_name, table);
        }

        Ok(())
    }

    pub fn delete_table(&mut self, name: String) -> Result<(), DatabaseError> {
        if self.tables.remove(&name).is_none() {
            return Err(DatabaseError::TableNotFound { name });
        }
        Ok(())
    }

    /// Gets an immutable reference to a table.
    pub fn get_table(&self, name: String) -> Result<&Table, DatabaseError> {
        self.tables
            .get(&name)
            .ok_or(DatabaseError::TableNotFound { name })
    }

    /// Mutable version if needed:
    pub fn get_table_mut(&mut self, name: String) -> Result<&mut Table, DatabaseError> {
        self.tables
            .get_mut(&name)
            .ok_or(DatabaseError::TableNotFound { name })
    }
}


// ========================================================================================
// TESTS
// ========================================================================================
#[cfg(test)]
mod table_crud_tests {
    use crate::schema::{Schema};
    use crate::column::{Column, DataType};
    use crate::database::{Database, DatabaseError}; 
    use std::collections::HashMap;

    fn test_schema() -> Schema {
        Schema::new(vec![
            Column {
                name: "name".into(),
                data_type: DataType::String,
                constraints: HashMap::new(),
            },
            Column {
                name: "age".into(),
                data_type: DataType::Integer,
                constraints: HashMap::new(),
            },
        ])
        .unwrap()
    }

    #[test]
    fn test_create_table() {
        let mut db = Database::new();
        let result = db.create_table("users".to_string(), test_schema());
        assert!(result.is_ok());
        assert!(db.get_table("users".to_string()).is_ok());
    }

    #[test]
    fn test_update_table_name() {
        let mut db = Database::new();
        db.create_table("name".to_string(), test_schema()).unwrap();
        let result = db.update_table_name("name".to_string(), "new_name".to_string());
        assert!(result.is_ok());
        assert!(db.get_table("new_name".to_string()).is_ok());
        assert!(db.get_table("name".to_string()).is_err());
    }

    #[test]
    fn test_delete_table_successfully() {
        let mut db = Database::new();
        db.create_table("name".to_string(), test_schema()).unwrap();
        let result = db.delete_table("name".to_string());
        assert!(result.is_ok());
        assert!(db.get_table("name".to_string()).is_err());
    }

    #[test]
    fn test_create_table_duplicate_fails() {
        let mut db = Database::new();
        db.create_table("users".to_string(), test_schema()).unwrap();
        let result = db.create_table("users".to_string(), test_schema());
        assert_eq!(result, Err(DatabaseError::DuplicateTableName("users".to_string())));
    }

    #[test]
    fn test_update_table_name_not_found() {
        let mut db = Database::new();
        let result = db.update_table_name("missing".to_string(), "new".to_string());
        assert_eq!(result, Err(DatabaseError::TableNotFound { name: "missing".to_string() }));
    }

    #[test]
    fn test_delete_table_not_found() {
        let mut db = Database::new();
        let result = db.delete_table("nonexistent".to_string());
        assert_eq!(result, Err(DatabaseError::TableNotFound { name: "nonexistent".to_string() }));
    }
}
