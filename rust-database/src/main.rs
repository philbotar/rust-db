use rust_database::{database::Database, schema::{Schema}, column::{Column, ColumnBuilder, DataType, Constraint, ConstraintKind}, row::{Value}};

fn main() {
    // What are our Steps? 
    // 1. Create Database
    // 2. Create Columns in Schema. 
    // 3. Create Table with Columns 

    let mut database = Database::new();

    // Lets define our Columns.
    let mut columns = Vec::<Column>::new();

    // Define column data as tuples
    let column_data = [
        ("id", DataType::Integer, vec![
            Constraint::Unit(ConstraintKind::Unique),
            Constraint::Unit(ConstraintKind::NotNull),
        ]),
        ("name", DataType::String, vec![
            Constraint::Unit(ConstraintKind::NotNull),
        ]),
        ("email", DataType::String, vec![]),
    ];


    // Convert the data to Columns. 
    for (name, data_type, constraints) in &column_data {
        let mut builder = ColumnBuilder::new(name, data_type.clone());

        for constraint in constraints {
            builder = match constraint {
                Constraint::Unit(kind) => {
                    match kind {
                        ConstraintKind::NotNull => builder.not_null(),
                        ConstraintKind::Unique => builder.unique(),
                        ConstraintKind::Default => {
                            builder
                        }
                    }
                }

                Constraint::WithValue(kind, val) => {
                    match kind {
                        ConstraintKind::Default => builder.default(val.clone()).expect("Default value type mismatch"),
                        _ => builder,
                    }
                }
            }
        }

        let column = builder.build();
        columns.push(column);
    }


    // Okay, now we have our Columns. Lets make a schema. 
    let schema = Schema::new(columns);

    //Okay, Schema created, lets make the table in the database. 

    if let Err(e) = database.create_table("Table_1".to_string(), schema.unwrap()) {
        eprintln!("Failed to create table: {:?}", e);
        return;
    }

    let table = database.get_table_mut("Table_1".to_string()).unwrap();
    

    // We have the table, lets add some rows. We will just be passing in the vec<Values>. We must pass in each one. 
    let row = vec![
        Value::Integer(0),
        Value::String("Philip".to_string()),
        Value::String("philipbotar@gmail.com".to_string()),
    ];
    
    table.add_row(row);

    if let Some(row) = table.get_row(0) {
        println!("{:?}", row);
    } else {
        println!("Row with index 1 not found.");
    }

}