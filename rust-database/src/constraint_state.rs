use std::collections::{HashMap, HashSet, BTreeSet};
use crate::row::Value;
use crate::schema::Schema;


// ========================================================================================
// ENUM
// ========================================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstraintKind {
    NotNull,
    Unique,
    Default,
    Index,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Unit(ConstraintKind),         
    WithValue(ConstraintKind, Value), 
}

// ========================================================================================
// STRUCT
// ========================================================================================
#[derive(Debug, Default)]
pub struct ConstraintState {
    pub unique_values: HashMap<String, HashSet<Value>>,
    pub not_null_columns: HashSet<String>,
    pub default_values: HashMap<String, Value>,
    pub indexes: HashMap<String, BTreeSet<Value>>,

    // Composite unique: column group → seen combinations
    // pub composite_uniques: HashMap<Vec<String>, HashSet<Vec<Value>>>,

    // Foreign key enforcement: (referenced table, column) → allowed values
    // pub foreign_keys: HashMap<(String, String), HashSet<Value>>,
}

// ========================================================================================
// IMPLEMENTATIOn
// ========================================================================================

impl ConstraintState {
    pub fn new(_schema: &Schema) -> Self { 
        Self::from_schema(_schema)
    }

    pub fn from_schema(schema: &Schema) -> Self {
        let mut unique_values = HashMap::new();
        let mut default_values = HashMap::new();
        let mut not_null_columns = HashSet::new();
        let mut indexes = HashMap::new();

        for col in &schema.columns {
            for constraint in col.constraints.values() {
                match constraint {
                    Constraint::Unit(ConstraintKind::NotNull) => {
                        not_null_columns.insert(col.name.clone());
                    }
                    Constraint::Unit(ConstraintKind::Unique) => {
                        unique_values.insert(col.name.clone(), HashSet::new());
                    }
                    Constraint::Unit(ConstraintKind::Index) => {
                        indexes.insert(col.name.clone(), BTreeSet::new());
                    }
                    Constraint::WithValue(ConstraintKind::Default, val) => {
                        default_values.insert(col.name.clone(), val.clone());
                    }
                    _other => {
                        // TODO
                    }
                }
            }
        }

        return ConstraintState {
            unique_values,
            default_values,
            not_null_columns,
            indexes,
        }
    }
}



// ========================================================================================
// TESTS
// ========================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::Value;
    use crate::schema::{Schema};
    use crate::column::{Column, DataType};

    fn make_column(name: &str, data_type: DataType, constraints: Vec<Constraint>) -> Column {
        let mut map: HashMap<ConstraintKind, Constraint> = std::collections::HashMap::new();
        for c in constraints {
            match &c {
                Constraint::Unit(kind) => map.insert(*kind, c.clone()),
                Constraint::WithValue(kind, _) => map.insert(*kind, c.clone()),
            };
        }
        Column {
            name: name.to_string(),
            data_type, 
            constraints: map,
        }
    }
 
    fn make_schema(columns: Vec<Column>) -> Schema {
        return Schema::new(columns).unwrap();
    }

    #[test]
    fn test_constraint_state_from_schema_unique_and_not_null() {
        let col1 = make_column(
            "id",
            DataType::Integer,
            vec![
                Constraint::Unit(ConstraintKind::Unique),
                Constraint::Unit(ConstraintKind::NotNull),
            ],
        );
        let col2 = make_column(
            "name",
            DataType::String,
            vec![Constraint::Unit(ConstraintKind::NotNull)],
        );
        let schema = make_schema(vec![col1, col2]);

        let state = ConstraintState::from_schema(&schema);

        assert!(state.unique_values.contains_key("id"));
        assert!(state.not_null_columns.contains("id"));
        assert!(state.not_null_columns.contains("name"));
        assert!(!state.unique_values.contains_key("name"));
    }

    #[test]
    fn test_constraint_state_from_schema_default_value() {
        let col = make_column(
            "age",
            DataType::Integer,
            vec![Constraint::WithValue(
                ConstraintKind::Default,
                Value::Integer(42),
            )],
        );
        let schema = make_schema(vec![col]);

        let state = ConstraintState::from_schema(&schema);

        assert_eq!(
            state.default_values.get("age"),
            Some(&Value::Integer(42))
        );
    }

    #[test]
    fn test_constraint_state_empty_schema() {
        let schema = make_schema(vec![]);
        let state = ConstraintState::from_schema(&schema);

        assert!(state.unique_values.is_empty());
        assert!(state.not_null_columns.is_empty());
        assert!(state.default_values.is_empty());
    }

    #[test]
    fn test_constraint_state_multiple_constraints() {
        let col = make_column(
            "email",
            DataType::String,
            vec![
            Constraint::Unit(ConstraintKind::Unique),
            Constraint::Unit(ConstraintKind::NotNull),
            Constraint::WithValue(ConstraintKind::Default, Value::String("none@example.com".to_string())),
            ],
        );
        let schema = make_schema(vec![col]);

        let state = ConstraintState::from_schema(&schema);

        assert!(state.unique_values.contains_key("email"));
        assert!(state.not_null_columns.contains("email"));
        assert_eq!(
            state.default_values.get("email"),
            Some(&Value::String("none@example.com".to_string()))
        );
    }

    #[test]
    fn test_constraint_state_from_schema_index() {
        let col = make_column(
            "username",
            DataType::String,
            vec![
                Constraint::Unit(ConstraintKind::Index),
            ],
        );
        let schema = make_schema(vec![col]);

        let state = ConstraintState::from_schema(&schema);

        assert!(state.indexes.contains_key("username"));
        assert!(state.indexes.get("username").unwrap().is_empty());
    }

    #[test]
    fn test_constraint_state_index_and_other_constraints() {
        let col = make_column(
            "login",
            DataType::String,
            vec![
                Constraint::Unit(ConstraintKind::Index),
                Constraint::Unit(ConstraintKind::NotNull),
                Constraint::Unit(ConstraintKind::Unique),
            ],
        );
        let schema = make_schema(vec![col]);

        let state = ConstraintState::from_schema(&schema);

        assert!(state.indexes.contains_key("login"));
        assert!(state.not_null_columns.contains("login"));
        assert!(state.unique_values.contains_key("login"));
    }

}