use std::fs::OpenOptions;
use std::fs::File;
use std::fs;
use std::mem;
use std::io::{Read, Write};
use byteorder::{WriteBytesExt, BigEndian};
pub mod jrdb_type;
use jrdb_type::{
  JrDocument, 
  JrCollection, 
  JrString, 
  JrType, 
  JrAny, 
  AddGet,
  JrCondition,
  AddGetValue,
};

pub mod macros;

#[allow(dead_code)]
enum ActionType{
  Insert,
  Select,
  Update,
  UpdateForce,
  Delete,
}

pub struct HeaderDetail{
  found:bool,
  key:String,
  header_start:usize,
  content_start:usize,
  content_end:usize,
  content_size:usize,
  content_length:usize,
  content_type:u8,
  depth:u8,
}

#[allow(dead_code)]
struct Action{
  action_type:ActionType,
  from:String,
  keys:Vec<String>,
  condition:JrCondition,
  data:Vec<JrDocument>
}


pub struct Database{
  _file:File,
  data:Vec<u8>,
  file_name:String,
  actions:Vec<Action>
}

trait GenericDatabaseFeature<T>{
  fn insert(&self, from:&str, key:&str, data:T);
}

#[allow(dead_code)]
impl Database{
  
  /// Choose a database to open, will create if doesn't exist.
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// 
  /// fn main() {
  ///   //create "main.db" if file not exist
  ///   let db:Database = Database::from("main");
  /// }
  /// ```
  pub fn from(s:&str)->Database{
      
    if let Ok(db_file) = OpenOptions::new().read(true).write(true).open(format!("{}.db",s)){

      let mut db_file = db_file;
      let mut db_data = Vec::new();
      db_file.read_to_end(&mut db_data).unwrap();

      Database{
        _file:db_file,
        data:db_data,
        file_name:String::from(s),
        actions:vec![]
      }

    } else {

      let mut db_data = vec![0, 0, 4, 0, 0, 0, 11, 114, 111, 111, 116];
      let mut db_file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true).open(format!("{}.db",s)).unwrap();

      db_file.write(&mut db_data).unwrap();

      Database{
        _file:db_file,
        data:db_data,
        file_name:String::from(s),
        actions:vec![]
      }

    }

  }

  /// Execute the query
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// use jrdb::jrdb_type::JrDocument;
  /// 
  /// fn main() {
  ///   let db:Database = Database::from("main");
  /// 
  ///   //create JrDocument
  ///   let mut doc = JrDocument::new();
  ///   doc.add_value("name", String::from("Joel"));
  ///   doc.add_value("pass", String::from("ILoveErd"));
  ///   doc.add_value("age", 30);
  /// 
  ///   //create collection "users" if not exist
  ///   db.insert("users", doc).execute();
  /// }
  /// ```
  pub fn execute(&mut self)->JrCollection{
    let mut data:JrCollection = JrCollection::new();
    let mut actions = mem::replace(&mut self.actions, Vec::new());
    for elem in actions.iter_mut() {
      let action_type = &elem.action_type;
      if let ActionType::Insert = action_type{
        self.insert_action(elem);
      }else if let ActionType::Select = action_type{
        data = self.select_action(elem);
      }else if let ActionType::Update = action_type{
        self.update_action(elem);
      }else if let ActionType::Delete = action_type{
        self.delete_action(elem);
      }
    }

    data
  }

  /// Insert data into collection by provide a JrDocument and collection name.
  /// Collection will be created if not exist.
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// use jrdb::jrdb_type::JrDocument;
  /// 
  /// fn main() {
  ///   let db:Database = Database::from("main");
  /// 
  ///   //create JrDocument
  ///   let mut doc = JrDocument::new();
  ///   doc.add_value("name", String::from("Joel"));
  ///   doc.add_value("pass", String::from("ILoveErd"));
  ///   doc.add_value("age", 30);
  /// 
  ///   //create collection "users" if not exist
  ///   db.insert("users", doc).execute();
  /// }
  /// ```
  pub fn insert(&mut self, from:&str, doc:JrDocument)->&mut Self{
    self.actions.push(Action{
      action_type:ActionType::Insert,
      condition:cond_true!(),
      from:from.into(),
      keys:vec![],
      data:vec![doc]
    });
    self
  }

  /// Select data from database
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// use jrdb::jrdb_type::{ AddGetValue, JrCollection, JrDocument };
  /// 
  /// fn main(){
  ///   let db:Database = Database::from("main");
  /// 
  ///   //create JrDocument
  ///   let mut doc = JrDocument::new();
  ///   doc.add_value("name", String::from("Joel"));
  ///   doc.add_value("pass", String::from("ILoveErd"));
  ///   doc.add_value("age", 30);
  /// 
  ///   //create collection "users" if not exist
  ///   db.insert("users", doc).execute();
  ///   
  ///   //select all data from colection "users"
  ///   let collection: JrCollection = db.select("users").execute();
  ///   collection.print(0);
  /// }
  /// ```
  pub fn select(&mut self, from:&str)->&mut Self{
    self.actions.push(Action{
      action_type:ActionType::Select,
      condition:cond_true!(),
      from:from.into(),
      keys:vec![],
      data:vec![]
    });

    self
  }

  /// Update data from database
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// use jrdb::jrdb_type::{ AddGetValue, JrCollection, JrDocument };
  /// 
  /// fn main(){
  ///   let db:Database = Database::from("main");
  /// 
  ///   //create JrDocument
  ///   let mut doc = JrDocument::new();
  ///   doc.add_value("name", String::from("Joel"));
  ///   doc.add_value("pass", String::from("ILoveErd"));
  ///   doc.add_value("age", 30);
  /// 
  ///   //create collection "users" if not exist
  ///   db.insert("users", doc).execute();
  ///   
  ///   //select all data from collection "users"
  ///   let collection: JrCollection = db.select("users").execute();
  ///   collection.print(0);
  /// 
  ///   //create JrDocument for update
  ///   let mut updated_doc = JrDocument::new();
  ///   updated_doc.add_value("name", String::from("Mathew"));
  /// 
  ///   //update all data from collection "users"
  ///   db.update("users", updated_doc).execute();
  /// 
  ///   let collection: JrCollection = db.select("users").execute();
  ///   collection.print(0);
  /// }
  /// ```
  pub fn update(&mut self, from:&str, doc:JrDocument)->&mut Self{
    self.actions.push(Action{
      action_type:ActionType::Update,
      condition:cond_true!(),
      from:from.into(),
      keys:vec![],
      data:vec![doc]
    });
    self
  }

  /// Delete data from database
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// use jrdb::jrdb_type::{ AddGetValue, JrCollection, JrDocument };
  /// 
  /// fn main(){
  ///   let db:Database = Database::from("main");
  /// 
  ///   //create JrDocument
  ///   let mut doc = JrDocument::new();
  ///   doc.add_value("name", String::from("Joel"));
  ///   doc.add_value("pass", String::from("ILoveErd"));
  ///   doc.add_value("age", 30);
  /// 
  ///   //create collection "users" if not exist
  ///   db.insert("users", doc).execute();
  ///   
  ///   //select all data from collection "users"
  ///   let collection: JrCollection = db.select("users").execute();
  ///   collection.print(0);
  ///
  ///   //delete all data from collection "users"
  ///   db.delete("users").execute();
  /// 
  ///   let collection: JrCollection = db.select("users").execute();
  ///   collection.print(0);
  /// }
  /// ```
  pub fn delete(&mut self, from:&str)->&mut Self{
    self.actions.push(Action{
      action_type:ActionType::Delete,
      condition:cond_true!(),
      from:from.into(),
      keys:vec![],
      data:vec![]
    });
    self
  }

  /// Apply condition while select, delete or update the database
  /// 
  /// # Examples
  /// ```
  /// use jrdb::Database;
  /// use jrdb::jrdb_type::{ AddGetValue, JrCollection, JrDocument };
  /// use jrdb::exp;
  /// 
  /// fn main(){
  ///   let db:Database = Database::from("main");
  /// 
  ///   //create JrDocument
  ///   let mut doc = JrDocument::new();
  ///   doc.add_value("name", String::from("Joel"));
  ///   doc.add_value("pass", String::from("ILoveErd"));
  ///   doc.add_value("age", 30);
  /// 
  ///   //create collection "users" if not exist
  ///   db.insert("users", doc).execute();
  ///   
  ///   //select all data from collection "users"
  ///   let collection: JrCollection = 
  ///     db.select("users")
  ///     .condition(exp! {"name" ;== "'Mathew'"})
  ///     .execute();
  ///   
  ///   //nothing will show since no document with in 'users' with name 'Manthew'
  ///   collection.print(0);
  /// 
  ///   //select all data from collection "users"
  ///   let collection: JrCollection = 
  ///     db.select("users")
  ///     .condition(exp! {"name" ;== "'Joel'"})
  ///     .execute();
  ///   
  ///   //shows document with name 'Joel'
  ///   collection.print(0);
  /// }
  /// ```
  pub fn condition(&mut self, cond:JrCondition)->&mut Self{
    let i = self.actions.len();
    self.actions[i-1].condition = cond;
    self
  }

  fn insert_action(&mut self, action:&mut Action){
    //let action = self.actions[i];
    let mut header_detail = self.get_header_detail_by_pos(0);
    self.find_and_insert(&action.from, &mut header_detail, &mut action.data[0]);
    fs::write(format!("{}.db", &self.file_name),&self.data).unwrap();
    //self.execute();
  }

  fn select_action(&mut self, action:&mut Action)->JrCollection{
    let mut header_detail = self.get_header_detail_by_pos(0);
    let mut collection_header = self.get_by_key_from_doc(&mut header_detail, action.from.split('.').nth(0).unwrap(), 1);

    self.select_with_condition(&action.from, &mut header_detail,&mut collection_header, &action.condition)
  }

  fn update_action(&mut self, action:&mut Action){
    let mut header_detail = self.get_header_detail_by_pos(0);
    let mut collection_header = self.get_by_key_from_doc(&mut header_detail, action.from.split('.').nth(0).unwrap(), 1);
    self.update_with_condition(&action.from, &mut header_detail, &mut collection_header, &action.condition, &mut action.data[0]);
    fs::write(format!("{}.db", &self.file_name),&self.data).unwrap();
  }

  fn update_with_condition(
    &mut self, _from:&str, 
    parent:&mut HeaderDetail, target:&mut HeaderDetail, 
    condition:&JrCondition, doc:&mut JrDocument
  ){
    if target.found {
      let mut collect_size_added = 0;

      //start looping item in target one by one
      self.loop_item_from_bytes(parent, target, &mut |db, collect_parent, collect_target| {
        let mut jr_doc = JrDocument::new();

        db.loop_item_from_bytes(collect_parent, collect_target, &mut |db, _, doc_target| {
          db.add_content_by_header(&mut jr_doc, doc_target);
        });

        if condition.result(&jr_doc) {
          let mut doc_size_added = 0;
          let mut start_pos = collect_target.content_start;
          doc.loop_key(&mut |key, data|{
            let val = db.update_key_by_pos(
              key, 
              data, 
              start_pos, 
              collect_target
            );
            start_pos = val.0;
            doc_size_added += val.1;
          });
          collect_size_added += doc_size_added;
          collect_parent.content_size = (collect_parent.content_size as i64 + doc_size_added) as usize;
          collect_parent.content_end = collect_parent.content_size + collect_parent.header_start;

        }

      });

      self.update_size(
        target.header_start,
        target.content_size
      );

      let new_content_size = (parent.content_size as i64 + collect_size_added) as usize;

      self.update_size(
        parent.header_start,
        new_content_size
      );
      parent.content_size = new_content_size;
      parent.content_end = parent.header_start + new_content_size;
    }
  }

  fn update_key_by_pos(&mut self, key:&str, data:&mut JrAny, start:usize, target_header:&mut HeaderDetail)->(usize, i64){
    let header = self.get_pos_by_key(start, target_header.content_end, key, 255);

    let mut data_type = 255;
    let mut bytes_data = if let JrAny::JrI64(v) = data{
      data_type = 3;
      v.get_bytes(target_header.depth+1)
    }else if let JrAny::JrString(v) = data {
      data_type = 2;
      v.get_bytes(target_header.depth+1)
    }else{
      vec![]
    };

    let mut attr_header = self.new_attr_header(data_type, bytes_data.len() as u32 , key.into(), target_header.depth+1);
    attr_header.append(&mut bytes_data);

    let val = if header.found {
      self.append_data(header.header_start, header.content_end, &mut attr_header);
      (
        header.header_start + attr_header.len(), 
        attr_header.len() as i64 - header.content_size as i64
      )
    }else{
      self.append_data(target_header.content_end, target_header.content_end, &mut attr_header);
      (
        start, 
        attr_header.len() as i64
      )
    };

    let new_content_size = (target_header.content_size as i64 + val.1) as usize;

    self.update_size(
      target_header.header_start,
      new_content_size
    );

    target_header.content_size = new_content_size;
    target_header.content_end = target_header.header_start + new_content_size;

    val
  }

  fn delete_action(&mut self, action:&mut Action){
    let mut header_detail = self.get_header_detail_by_pos(0);
    let mut collection_header = self.get_by_key_from_doc(&mut header_detail, action.from.split('.').nth(0).unwrap(), 1);
    self.delete_with_condition(&action.from, &mut header_detail, &mut collection_header, &action.condition);
    fs::write(format!("{}.db", &self.file_name),&self.data).unwrap();
  }

  fn delete_with_condition(
    &mut self, _from:&str,parent:&mut HeaderDetail, target:&mut HeaderDetail, 
    condition:&JrCondition,
  ){
    if target.found {
      let mut collect_size_added = 0;
      self.loop_item_from_bytes(parent, target, &mut |db, collect_parent, collect_target| {
        let mut jr_doc = JrDocument::new();

        db.loop_item_from_bytes(collect_parent, collect_target, &mut |db, _, doc_target| {
          db.add_content_by_header(&mut jr_doc, doc_target);
        });
        if condition.result(&jr_doc) {
          
          let doc_size_added = db.delete_key_by_header(collect_target);
          collect_size_added += doc_size_added;
          collect_parent.content_size = (collect_parent.content_size as i64 + doc_size_added) as usize;
          collect_parent.content_end = collect_parent.content_size + collect_parent.header_start;
        }
      });

      self.update_size(
        target.header_start,
        target.content_size
      );
  
      let new_content_size = (parent.content_size as i64 + collect_size_added) as usize;

      self.update_size(
        parent.header_start,
        new_content_size
      );
      parent.content_size = new_content_size;
      parent.content_end = parent.header_start + new_content_size;
    }
  }

  fn delete_key_by_header(&mut self, header:&mut HeaderDetail)->i64{
    let mut data = vec![];
    self.append_data(header.header_start, header.content_end, &mut data);
    header.content_size = 0;
    return header.header_start as i64 - header.content_end as i64
  }

  fn add_content_by_header(&mut self, jr_doc:&mut JrDocument, doc_target:&mut HeaderDetail){
    if doc_target.content_type == 2 {
      let _:String = jr_doc.get_value_from_db(self, doc_target);
    } else if doc_target.content_type == 3 {
      let _:i64 = jr_doc.get_value_from_db(self, doc_target);
    }
  }

  fn select_with_condition(&mut self, _from:&str, parent:&mut HeaderDetail, target:&mut HeaderDetail, condition:&JrCondition)->JrCollection{

    let mut jr_collec = JrCollection::new();
    if target.found {
      //this loop the collection found
      self.loop_item_from_bytes(parent, target, &mut |db, collect_parent, collect_target| {
        let mut jr_doc = JrDocument::new();

        let id = JrString::new( format!("{}",collect_target.key) );
        jr_doc.add("_id", id);
        // this loop throught the key in the item
        db.loop_item_from_bytes(collect_parent, collect_target, &mut |db, _, doc_target| {
          db.add_content_by_header(&mut jr_doc, doc_target);
        });
        if condition.result(&jr_doc) {
          jr_collec.add(jr_doc);
        }
      })
    }
    jr_collec
  }

  fn get_bytes_content(&self, from:&HeaderDetail)->Vec<u8>{
    self.data[from.content_start..from.content_end].to_vec()
  }

  fn get_by_key_from_doc(&mut self, from:&HeaderDetail, target_key:&str, target_type: u8)->HeaderDetail{
    self.get_pos_by_key(from.content_start, from.content_end, target_key, target_type)
  }


  fn loop_item_from_bytes<F>(&mut self, _parent:&mut HeaderDetail, target:&mut HeaderDetail, f:&mut F) where
  F:FnMut(&mut Database, &mut HeaderDetail, &mut HeaderDetail) 
  {
    let mut curr_pos = target.content_start;
    while curr_pos != target.content_end {
      let mut document_header = self.get_header_detail_by_pos(curr_pos);
      f(self, target, &mut document_header);
      curr_pos += document_header.content_size;
    }
  }

  fn find_and_insert(&mut self, from:&str, pos:&mut HeaderDetail, doc:&mut JrDocument)->usize{
    let mut total_bytes_added = 0;
    let mut data = from.split(".");
    
    let mut collection_header = self.get_pos_by_key(pos.content_start, pos.content_end, data.nth(0).unwrap() , 1);
    
    if collection_header.found {
      total_bytes_added += self.append_to_collec_bytes_end(&mut collection_header, pos, doc);
    }else{
      let mut header = self.new_attr_header(
        1,
        0,
        format!("{}", collection_header.key),
        pos.depth+1
      );
      let new_arr_start = pos.content_end;
      total_bytes_added += self.append_to_doc_bytes_end(pos, &mut header);
      let mut collection_pos = self.get_header_detail_by_pos(new_arr_start);
      total_bytes_added += self.append_to_collec_bytes_end(&mut collection_pos, pos, doc);
    }

    total_bytes_added
  }

  fn append_to_collec_bytes_end(&mut self, collection_pos:&mut HeaderDetail, parent_pos:&mut HeaderDetail, doc:&mut JrDocument)->usize{
    let mut total_bytes_added = 0;
    let mut total_added = 0;

    total_added += 1;

    let id = collection_pos.content_length + total_added;
    let mut content = doc.get_bytes(collection_pos.depth+1);
    let mut header = self.new_attr_header(0, content.len() as u32, format!("{}",id), collection_pos.depth+1);
    header.append(&mut content);

    let len = header.len();
    total_bytes_added += len;

    self.append_data(
      collection_pos.content_end, 
      collection_pos.content_end, 
      &mut header
    );

    self.update_size(
      collection_pos.header_start, 
      collection_pos.content_size + total_bytes_added
    );

    self.update_len(
      collection_pos.header_start,
      collection_pos.content_length + total_added
    );

    collection_pos.content_size += total_bytes_added;
    collection_pos.content_end += total_bytes_added;
    collection_pos.content_length += total_added;

    self.update_size(
      parent_pos.header_start, 
      parent_pos.content_size + total_bytes_added
    );
    parent_pos.content_size += total_bytes_added;
    parent_pos.content_end += total_bytes_added;

    total_bytes_added
  }

  fn append_to_doc_bytes_end(&mut self, pos:&mut HeaderDetail, data:&mut Vec<u8>)->usize{
    let data_len = data.len();
    self.append_data(
      pos.content_end, 
      pos.content_end, 
      data
    );
    self.update_size(
      pos.header_start, 
      pos.content_size + data_len
    );
    pos.content_size += data_len;
    pos.content_end += data_len;
    data_len
  }



  fn update_size(&mut self, start_pos:usize, len:usize){
    let mut bytes_data = self.u32_to_4bytes_arr(len as u32);
    self.append_data(start_pos+3, start_pos+7, &mut bytes_data)
  }

  fn update_len(&mut self, start_pos:usize, len:usize){
    let mut bytes_data = self.u32_to_4bytes_arr(len as u32);
    self.append_data(start_pos+7, start_pos+11, &mut bytes_data)
  }

  fn u32_to_4bytes_arr(&self, value:u32)->Vec<u8>{
    let mut attr_size_bytes:Vec<u8> = Vec::new();
    let slice_u32: &[u32] = &*vec![value];
    for &n in slice_u32 {
      let _ = attr_size_bytes.write_u32::<BigEndian>(n);
    }
    attr_size_bytes
  }

  fn to_4bytes_arr(&self, slice:&[u8])->[u8;4]{
    let mut arr:[u8;4] = [0;4];
    for (i,item) in slice.iter().enumerate() {
      arr[i] = *item;
    }
    arr
  }

  fn to_8bytes_arr(&self, slice:&[u8])->[u8;8]{
    let mut arr:[u8;8] = [0;8];
    for (i,item) in slice.iter().enumerate() {
      arr[i] = *item;
    }
    arr
  }

  fn get_pos_by_key(&mut self, start_pos:usize, limit:usize, target_key:&str,target_type:u8)->HeaderDetail{

    if start_pos == limit {
      return HeaderDetail{
        found:false,
        key:String::from(target_key),
        header_start:0,
        content_start:0,
        content_end:0,
        content_length:0,
        content_size:0,
        content_type:target_type,
        depth:0
      }
    }

    let header_detial = self.get_header_detail_by_pos(start_pos);

    if header_detial.content_type != target_type && target_type != 255{
      self.get_pos_by_key(header_detial.content_end, limit, target_key, target_type)
    }else{
      if header_detial.key.eq(&String::from(target_key)){
        header_detial
      }else{
        self.get_pos_by_key(header_detial.content_end, limit, target_key, target_type)
      }
    }
    
  }

  fn get_header_detail_by_pos(&self, start_pos:usize)->HeaderDetail{
    let key_len = self.data[start_pos+2];
    let content_type = self.data[start_pos+1];
    let depth = self.data[start_pos];
    let key_start_pos = if content_type!=1{
      start_pos+7
    }else{
      start_pos+11
    };

    let key_end_pos = key_start_pos+(key_len as usize);
    let key_utf8_bytes = self.data[key_start_pos..key_end_pos].to_vec();
    let key = String::from_utf8(key_utf8_bytes).unwrap();
    let buffer:[u8;4] = self.to_4bytes_arr(&self.data[(start_pos+3)..(start_pos+7)]);
    let content_size = u32::from_be_bytes(buffer);
    let content_end = start_pos+content_size as usize;
    let mut content_length = 0;
    if content_type == 1{
      let buffer:[u8;4] = self.to_4bytes_arr(&self.data[(start_pos+7)..(start_pos+11)]);
      content_length = u32::from_be_bytes(buffer) as usize;
    }

    HeaderDetail{
      found:true,
      key,
      header_start:start_pos,
      content_start:key_end_pos,
      content_end,
      content_length,
      content_size:content_size as usize,
      content_type,
      depth
    }
  }

  fn new_attr_header(&self, data_type:u8,size:u32, name:String, depth:u8)->Vec<u8>{
    let mut header:Vec<u8> = [depth,data_type,0].to_vec();
    
    let mut key = String::from(name).into_bytes();

    let key_len = key.len() as u8;
    header[2] = key_len;
    let attr_size = if data_type==1 {
      11+key_len
    }else{
      7+key_len
    } as u32;

    let mut attr_size_bytes:Vec<u8> = Vec::new();
    let slice_u32: &[u32] = &*vec![attr_size+size];
    for &n in slice_u32 {
      let _ = attr_size_bytes.write_u32::<BigEndian>(n);
    }

    header.append(&mut attr_size_bytes);
    
    let mut len = [0,0,0,0].to_vec();
    if data_type==1{
      header.append(&mut len);
    }
    header.append(&mut key);
    header
  }

  fn append_data(&mut self, start_pos:usize, end_pos:usize, data:&mut Vec<u8>){
    self.data.splice(start_pos..end_pos, data.iter().cloned());
  }
}