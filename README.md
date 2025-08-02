# Rust DB

**`rust-db`** is a from‑scratch implementation of a **relational database engine** written in pure Rust, using only the Rust standard library.
It’s a project designed to explore **how relational database management systems (RDBMS)** work under the hood — from **schemas** and **row storage** to **indexing**, **constraints**, and **persistence**.

This is **not** a wrapper over SQLite or an ORM. This is a ground‑up exploration of database internals, intentionally avoiding third‑party crates (other than `std`) to deeply understand how these systems are built.

# How Relational Databases work under the hood

What do we generally know about Relational Database systems such as MySQL?

- A database is a collection of Tables
- Data is presented as a collection of Tables, each of which have rows and columns
- Users have the ability to manipulate data in tabular form.

So from what we know, how can we use this information to build out DB from scratch ?

- We need to have the ability to have a variable amount of rows and columns.
- We need a key for each row. This key will be unique identifier for each row in a table. this means we should start with a HashMap

* [x] Be able to create a schema
* [x] Be able to create a database
* [x] Be able to create Tables. Each table has a name, schema and metadata against it.
* [x] Be able to add rows to a table
* [ ] Add ability to have column constraints
* [ ] Add "Check" Constraint (Expressions)
* [ ] Add the ability to have foreign_keys.
* [ ] Persist the data locally.
* [ ] Be able to perform queries on the data
* [ ] Add ability to have indexes against the data

# What we are allowing ourselves to use?

As this is an exercise for me to learn more about how databases work and how i can apply Rust in a way thats a bit more in-depth, im taking the challenge of only using the standard library.
