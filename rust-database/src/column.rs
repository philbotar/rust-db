use crate::row::Value;
use std::collections::{HashMap};
use crate::constraint_state::{ConstraintKind, Constraint};

// ==============================================================================
// ENUMS
// ==============================================================================
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    String,
    Integer,
    Null,
}

#[derive(Debug, PartialEq)]
pub enum ColumnError {
    DefaultValueTypeMismatch,
}


// ==============================================================================
// STRUCTS
// ==============================================================================
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub constraints: HashMap<ConstraintKind, Constraint>,
}

// The builder is now more powerful. It builds up the constraints internally.
#[derive(Debug)]
pub struct ColumnBuilder {
    name: String,
    data_type: DataType,
    constraints: HashMap<ConstraintKind, Constraint>,
}


// ==============================================================================
// IMPLEMENTATIONS
// ==============================================================================
impl ColumnBuilder {
    pub fn new(name: &str, data_type: DataType) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            constraints: HashMap::new(),
        }
    }

    pub fn not_null(mut self) -> Self {
        self.constraints.insert(ConstraintKind::NotNull, Constraint::Unit(ConstraintKind::NotNull));
        self
    }

    pub fn unique(mut self) -> Self {
        self.constraints.insert(ConstraintKind::Unique, Constraint::Unit(ConstraintKind::Unique));
        self
    }

    pub fn default(mut self, value: Value) -> Result<Self, ColumnError> {
        if value.get_data_type() != self.data_type {
            return Err(ColumnError::DefaultValueTypeMismatch);
        }

        self.constraints.insert(
            ConstraintKind::Default,
            Constraint::WithValue(ConstraintKind::Default, value),
        );

        Ok(self)
    }

    pub fn index(mut self) -> Self {
        self.constraints.insert(ConstraintKind::Index, Constraint::Unit(ConstraintKind::Index));
        self
    }

    pub fn build(self) -> Column {
        Column {
            name: self.name,
            data_type: self.data_type,
            constraints: self.constraints,
        }
    }
}


// ==============================================================================
// TESTS
// ==============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::Value;

    #[test]
    fn test_fluent_builder_with_single_constraints() {
        let column = ColumnBuilder::new("id", DataType::Integer)
            .not_null()
            .unique()
            .build();

        assert!(column.constraints.contains_key(&ConstraintKind::NotNull));
        assert!(column.constraints.contains_key(&ConstraintKind::Unique));

        assert_eq!(
            column.constraints.get(&ConstraintKind::NotNull),
            Some(&Constraint::Unit(ConstraintKind::NotNull))
        );
        assert_eq!(
            column.constraints.get(&ConstraintKind::Unique),
            Some(&Constraint::Unit(ConstraintKind::Unique))
        );
    }

    #[test]
    fn test_builder_with_valid_default() {
        let result = ColumnBuilder::new("age", DataType::Integer)
            .default(Value::Integer(18));

        assert!(result.is_ok());
        let column = result.unwrap().build();

        assert!(column.constraints.contains_key(&ConstraintKind::Default));
        assert_eq!(
            column.constraints.get(&ConstraintKind::Default),
            Some(&Constraint::WithValue(ConstraintKind::Default, Value::Integer(18)))
        );
    }

    #[test]
    fn test_builder_with_invalid_default() {
        let result = ColumnBuilder::new("age", DataType::Integer)
            .default(Value::String("eighteen".to_string()));

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, ColumnError::DefaultValueTypeMismatch);
    }

    #[test]
    fn test_overwriting_default_value() {
        let column = ColumnBuilder::new("status", DataType::Integer)
            .default(Value::Integer(0)).unwrap()
            .default(Value::Integer(1)).unwrap() // This one should win
            .build();

        assert_eq!(column.constraints.len(), 1);

        assert_eq!(
            column.constraints.get(&ConstraintKind::Default),
            Some(&Constraint::WithValue(ConstraintKind::Default, Value::Integer(1)))
        );
    }

    
    #[test]
    fn test_adding_index() {
        let column = ColumnBuilder::new("status", DataType::Integer).index().build();

        assert_eq!(column.constraints.len(), 1);
        assert!(column.constraints.contains_key(&ConstraintKind::Index));
        assert_eq!(
            column.constraints.get(&ConstraintKind::Index),
            Some(&Constraint::Unit(ConstraintKind::Index))
        );
    }
}
