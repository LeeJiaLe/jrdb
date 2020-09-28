# rust-jrdb
Joel Rust Database, a document oriented NoSql database created for learning purpose

Currently only support insert and select all function. Nested document/ collection not yet support.

Example usage:
```rust
mod jrdb;

extern crate byteorder;
use jrdb::Database;
use jrdb::jrdb_type::{ JrCollection, JrString, JrDocument, AddGet };


fn main(){
  //create file "main.db" if not exist
  let mut db:Database = Database::from("main");

  let mut doc:JrDocument = JrDocument::new();
  doc.add("name", JrString::new("Joel".into()));
  doc.add("pass", JrString::new("ILoveErd".into()));

  //create collection "user" if not exist
  db.insert("user",&mut doc);

  let collection:JrCollection = db.select("user");

  let first_doc:&JrDocument = collection.get(0);

  let name:&JrString = first_doc.get("name").unwrap();
  let pass:&JrString = first_doc.get("pass").unwrap();

  //should print Joel
  println!("{}",name.get());

  //should print ILoveErd
  println!("{}",pass.get());
}

```

## Next Scope
- Select specific key
- Select with condition
- "Update" feature
- "Delete" feature
- Create document (Maybe)
