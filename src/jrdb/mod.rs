use std::fs::OpenOptions;
use std::fs::File;
use std::fs;
use std::io::{Read, Write};
use byteorder::{WriteBytesExt, BigEndian};
pub mod jrdb_type;

use jrdb_type::{JrDocument, JrCollection, JrString, JrType, AddGet};


struct HeaderDetail{
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

// use super::jrdb_type::JrCollection;
pub struct Database{
  _file:File,
  pub data:Vec<u8>,
  file_name:String
}

trait GenericDatabaseFeature<T>{
  fn insert(&self, from:&str, key:&str, data:T);
}

#[allow(dead_code)]
impl Database{
  pub fn from(s:&str)->Database{
      
    if let Ok(db_file) = OpenOptions::new().read(true).write(true).open(format!("{}.db",s)){

      let mut db_file = db_file;
      let mut db_data = Vec::new();
      db_file.read_to_end(&mut db_data).unwrap();

      Database{
        _file:db_file,
        data:db_data,
        file_name:String::from(s),
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
      }

    }

  }

  pub fn insert(&mut self, from:&str, doc:&mut JrDocument)->&mut Self{
    let mut header_detail = self.get_header_detail_by_pos(0);
    self.find_and_insert(from, &mut header_detail, doc);
    self.execute();
    self
  }

  pub fn select(&mut self, from:&str)->JrCollection{
    let mut header_detail = self.get_header_detail_by_pos(0);
    let mut collection_header = self.get_by_key_from_doc(&mut header_detail, from.split('.').nth(0).unwrap(), 1);

    self.select_with_condition(from, &mut header_detail,&mut collection_header, Vec::new())
  }

  fn select_with_condition(&mut self, _from:&str, parent:&mut HeaderDetail, target:&mut HeaderDetail, _condition:Vec<(&str,&str,&str)>)->JrCollection{

    let mut jr_collec = JrCollection::new();
    if target.found {
      //this loop the collection found
      self.loop_item_from_bytes(parent, target, &mut |collect_parent, collect_target| {
        let mut jr_doc = JrDocument::new();

        let id = JrString::new( format!("{}",collect_target.key) );
        jr_doc.add("_id", id);
        // this loop throught the key in the item
        self.loop_item_from_bytes(collect_parent, collect_target, &mut |_, doc_target| {
          
          if doc_target.content_type == 2{
            let data = match String::from_utf8(self.get_bytes_content(doc_target)){
              Ok(d)=>d,
              Err(_)=>String::from(""),
            };
            let value = JrString::new(data);
            jr_doc.add(&doc_target.key, value);
          }
        });
        
        jr_collec.add(jr_doc);
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


  fn loop_item_from_bytes<F>(&self, _parent:&mut HeaderDetail, target:&mut HeaderDetail, f:&mut F) where
  F:FnMut(&mut HeaderDetail, &mut HeaderDetail) 
  {
    let mut curr_pos = target.content_start;
    while curr_pos != target.content_end {
      let mut document_header = self.get_header_detail_by_pos(curr_pos);
      f(target, &mut document_header);
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

  pub fn execute(&mut self){
    fs::write(format!("{}.db", &self.file_name),&self.data).unwrap();
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

  fn get_pos_by_key(&mut self, start_pos:usize, limit:usize, target_key:&str,target_type:u8)->HeaderDetail{

    if start_pos == self.data.len() {
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

    if header_detial.content_type != target_type{
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
    
    //fs::write(format!("{}.db", &self.file_name),&self.data).unwrap(); 
    //self.file.write(&self.data).unwrap();
  }

  // pub fn clear_db(&mut self){
  //     self.data = Vec::new();
  //     &self.file.write(&self.data);
  // }
}

// impl GenericDatabaseFeature<JrCollection> for Database{
//   fn insert(&self, from:&str, key:&str, data:JrCollection){
    
//   }
// }