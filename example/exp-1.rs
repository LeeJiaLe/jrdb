use jrdb::jrdb_type::{AddGetValue, JrCollection, JrDocument};
use jrdb::Database;

use jrdb::{exp, jr_doc};

fn main() {
  //create file "main.db" if not exist
  let mut db: Database = Database::from("main");

  //use macro to create JrDocument
  let doc = jr_doc! {
    "name"; String => "Mathew".into(),
    "pass"; String => "ILoveERD".into(),
    "age"; i64 => 400,
  };

  //basic way to create JrDocument
  let mut doc2 = JrDocument::new();
  doc2.add_value("name", String::from("Joel"));
  doc2.add_value("pass", String::from("ILoveErd"));
  doc2.add_value("age", 30);

  //create collection "users" and admin if not exist
  db.insert("users", doc)
    .insert("admins", doc2)
    .execute();

  //update "users" with condition
  db.update(
    "users",
    jr_doc! {
      "name"; String => "Jason".into()
    },
  )
  .condition(exp! {"name" ;== "'Joel'"})
  .execute();

  //select "users"
  let collection: JrCollection = db.select("users").execute();
  collection.print(0);

  //select "admins"
  let collection: JrCollection = db.select("admins").execute();
  collection.print(0);

  //delete with condition
  db.delete("users")
    .condition(exp! {"name" ;== "'Jason'"})
    .execute();

  let collection: JrCollection = db.select("users").execute();
  collection.print(0);
}
