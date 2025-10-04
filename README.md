# Rust DB

**`rust-db`** is a from‑scratch implementation of a **relational database engine** written in pure Rust, using only the Rust standard library.
It’s a project designed to explore **how relational database management systems (RDBMS)** work under the hood — from **schemas** and **row storage** to **indexing**, **constraints**, and **persistence**.

This is **not** a wrapper over SQLite or an ORM. This is a ground‑up exploration of database internals, intentionally avoiding third‑party crates (other than `std`) to deeply understand how these systems are built.

# How Relational Databases work under the hood

What do we generally know about Relational Database systems such as MySQL?

- A database is a collection of Tables
- Data is presented as a collection of Tables, each of which have rows and columns
- Users have the ability to manipulate data in tabular form.

# How are we designing this?

## Core Components

- database.rs
  - create_table
  - update_table_name
  - delete_table
- table.rs
  - add_row
  - delete_row
  - update_row
  - query_rows
  - update_schema
- row.rs
  - constructor
  - validate_row
- column.rs
  - not_null
  - unique
  - default
  - constructor
- schema.rs
  - validate_schema

# Our To-Do

- [X] Be able to create a schema
- [X] Be able to create a database
- [X] Be able to create Tables. Each table has a name, schema and metadata against it.
- [X] Be able to add rows to a table
- [X] Add ability to have column constraints
- [ ] Apply Constraints to each row.
- [ ] Add "Check" Constraint (Expressions)
- [ ] Add the ability to have foreign_keys.
- [ ] Persist the data locally.
- [ ] Create a Tokenizer to split Queries into tokens
- [ ] Create a Parser to convert tokens to an Abstract Syntax Tree
- [ ] Be able to perform queries on the data
- [ ] Add ability to have indexes against the data
- [ ] Add ability to Partition the data(?)

# What we are allowing ourselves to use?

As this is an exercise for me to learn more about how databases work and how i can apply Rust in a way thats a bit more in-depth, im taking the challenge of only using the standard library.
