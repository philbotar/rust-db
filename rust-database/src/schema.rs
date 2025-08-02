
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType { 
    String,
    Integer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstraintType {
    NotNull,
    Unique,
    Default(DataType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
}

#[derive(Clone)]
pub struct Column { 
    pub name: String,
    pub data_type: DataType,
    pub index: usize,
    pub constraints: Option<HashSet<Constraint>>,
}

#[derive(Clone)]
pub struct Schema { 
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn new(columns: Vec<Column>) -> Self {
        let schema = Schema { columns };

        if !Schema::schema_validator(schema.clone()) {
            panic!("Schema validation failed");
        }

        schema
    }

    // Simple collision check. See if theres a column in that location, and that the names are unique. 
    fn schema_validator(schema: Schema) -> bool {
        let mut columns_used = vec![false; schema.columns.len()];
        let mut unique_names = HashSet::<String>::new();

        for (i, column) in schema.columns.iter().enumerate() {
            Self::create_constraints_validator(column);

            if columns_used[column.index] {
                panic!("Duplicate column index found: {}", i);
            } else if unique_names.contains(&column.name)  {
                panic!("Duplicate column name found: {}", column.name)
            } else {
                columns_used[column.index] = true;
                unique_names.insert(column.name.to_string());
            }
        }

        true
    }


    fn create_constraints_validator(column: &Column) {
        if let Some(constraints) = &column.constraints {
            for constraint in constraints {
                if let ConstraintType::Default(default_type) = &constraint.constraint_type {
                    if column.data_type != *default_type {
                        panic!(
                            "Default constraint type mismatch for column '{}': column type is {:?}, default type is {:?}",
                            column.name, column.data_type, default_type
                        );
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod schema_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate column index found")]
    fn test_duplicate_index() {
        let columns = vec![
            Column { name: "id".to_string(), data_type: DataType::Integer, index: 0, constraints: None },
            Column { name: "name".to_string(), data_type: DataType::String, index: 0, constraints: None }, 
        ];
        Schema::new(columns);
    }

    #[test]
    #[should_panic(expected = "Duplicate column name found")]
    fn test_duplicate_name() {
        let columns = vec![
            Column { name: "id".to_string(), data_type: DataType::Integer, index: 0, constraints: None },
            Column { name: "id".to_string(), data_type: DataType::String, index: 1, constraints: None }, 
        ];
        Schema::new(columns);
    }

    #[test]
    fn test_valid_schema() {
        let columns = vec![
            Column { name: "id".to_string(), data_type: DataType::Integer, index: 0, constraints: None },
            Column { name: "name".to_string(), data_type: DataType::String, index: 1, constraints: None },
        ];
        let schema = Schema::new(columns);
        assert_eq!(schema.columns.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Default constraint type mismatch for column")]
    fn test_default_constraint_type_mismatch() {
        let constraint = Constraint { constraint_type: ConstraintType::Default(DataType::String) };
        let constraints = Some(HashSet::from([constraint]));
        let columns = vec![
            Column { name: "id".to_string(), data_type: DataType::Integer, index: 0, constraints },
        ];
        Schema::new(columns);
    }

    #[test]
    fn test_valid_default_constraint() {
        let constraint = Constraint { constraint_type: ConstraintType::Default(DataType::String) };
        let constraints = Some(HashSet::from([constraint]));
        let columns = vec![
            Column { name: "name".to_string(), data_type: DataType::String, index: 0, constraints },
        ];
        let schema = Schema::new(columns);
        assert_eq!(schema.columns.len(), 1);
    }

    #[test]
    fn test_multiple_valid_constraints() {
        let constraints = Some(HashSet::from([
            Constraint { constraint_type: ConstraintType::NotNull },
            Constraint { constraint_type: ConstraintType::Unique },
            Constraint { constraint_type: ConstraintType::Default(DataType::Integer) },
        ]));
        
        let columns = vec![
            Column { name: "id".to_string(), data_type: DataType::Integer, index: 0, constraints },
        ];

        let schema = Schema::new(columns);
        assert_eq!(schema.columns.len(), 1);
    }
}


