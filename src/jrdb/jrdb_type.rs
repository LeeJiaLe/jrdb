use std::collections::BTreeMap;
use byteorder::{WriteBytesExt, BigEndian};
use std::fmt::{ Display, Formatter };
use std::fmt;
use super::HeaderDetail;
use super::Database;

#[derive(Clone)]
pub enum JrAny{
  JrCollection(JrCollection),
  JrDocument(JrDocument),
  JrString(JrString),
  JrI64(JrI64),
}

#[derive(Clone)]
pub enum ConditionType{
  And,
  Or,
  Eq,
  NEq,
  Gt,
  NGt,
  GtE,
  NGtE,
  St,
  NSt,
  StE,
  NStE,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct JrCondition{
  cond_type:ConditionType,
  conditions:Vec<JrCondition>,
  expression:(String, String)
}

impl JrCondition{
  pub fn and()->Self{
    JrCondition{
      cond_type:ConditionType::And,
      conditions:vec![],
      expression:("".into(),"".into())
    }
  }

  pub fn or()->Self{
    JrCondition{
      cond_type:ConditionType::Or,
      conditions:vec![],
      expression:("".into(),"".into())
    }
  }

  pub fn new_exp(cond_type:ConditionType, conditions:Vec<JrCondition>, expression:(String, String))->Self{
    JrCondition{
      cond_type,
      conditions,
      expression
    }
  }

  pub fn add_cond(&mut self, cond:JrCondition){
    self.conditions.push(cond);
  }

  pub fn compare_string(&self, val1:&String, val2:&String)->bool{
    if let ConditionType::Eq = self.cond_type {
      val1.eq(val2)
    }else{
      false
    }
  }

  fn compare_i64(&self, val1:i64, val2:i64)->bool{
    if let ConditionType::Eq = self.cond_type {
      val1==val2
    }else{
      false
    }
  }

  fn get_as_string(&self, val:JrAny)->Option<String>{
    if let JrAny::JrString(s) = val{
      Some(s.get().clone())
    }else if let JrAny::JrI64(s) = val{
      Some(s.get().to_string())
    }else{
      None
    }
  }

  fn get_value(&self, value:&String, doc:&JrDocument)->Option<JrAny>{
    //horrible code, need fix (use regex)
    let mut r_chars = value.chars();
    let r_char1 = r_chars.nth(0).unwrap();
    let r_char2 = r_chars.nth(value.len()-2).unwrap();

    if r_char1.eq(&'\'') && r_char1.eq(&r_char2) {
      let mut val2:&str = &self.expression.1.clone();
      val2 = &val2[1..val2.len()];
      val2 = &val2[0..val2.len()-1];
      Some(JrAny::JrString(JrString::new(val2.into())))
    //assume is number
    }else if value.parse::<i64>().is_ok(){
      Some(JrAny::JrI64(JrI64::new(value.parse::<i64>().unwrap())))
    }else{
      if let Ok(v) = doc.get_value(&value){
        Some(JrAny::JrI64(JrI64::new(v)))
      }else if let Ok(v) = doc.get_value(&value){
        Some(JrAny::JrString(JrString::new(v)))
      }else{
        None
      }
    }
  }

  pub fn result(&self, doc:&JrDocument)->bool{
    if let ConditionType::And = self.cond_type {
      let mut data = true;
      for elem in self.conditions.iter() {
        if !elem.result(doc) {
          data = false;
          break
        }
      }
      data
    } else if let ConditionType::Or = self.cond_type {
      let mut data = false;
      for elem in self.conditions.iter() {
        if elem.result(doc) {
          data = true;
          break
        }
      }
      data
    }else{
      let mut result = false;

      let val1 = self.get_value(&self.expression.0, doc);
      let val2 = self.get_value(&self.expression.1, doc);

      if val1.is_some() && val2.is_some() {
        let left = val1.unwrap();
        let right = val2.unwrap();

        if let JrAny::JrString(s1) = left{
          if let Some(s2) = self.get_as_string(right){
            result = self.compare_string(s1.get() , &s2);
          }
        }else if let JrAny::JrString(s1) = right{
          if let Some(s2) = self.get_as_string(left){
            result = self.compare_string(s1.get() , &s2);
          }
        }else if let JrAny::JrI64(v1) = left {
          if let JrAny::JrI64(v2) = right {
            result = self.compare_i64(*v1.get(), *v2.get());
          }
        }
      }else{
        result = false;
      }
      result
    }
  }
}

pub trait AddGet<T>
where T:JrType
{
  fn add(&mut self, key:&str, item: T);
  fn get(&self, key:&str)->Result<&T, &str>;
}

pub trait AddGetValue<T>
{
  fn add_value(&mut self, key:&str, item: T);
  fn get_value(&self, key:&str)->Result<T, &str>;
  fn get_value_from_db(&mut self, db:&mut Database, header:&mut HeaderDetail)->T;
}

pub trait JrType{
  fn get_bytes(&mut self, depth:u8)->Vec<u8>;

  fn new_attr_header(data_type:u8, depth:u8, content_size:u32, name:String)->Vec<u8>{
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
    let slice_u32: &[u32] = &*vec![attr_size + content_size];
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
}

#[derive(Clone)]
pub struct JrI64{
  data:i64
}

impl JrI64{
  pub fn new(data:i64)->Self{
    JrI64{
      data
    }
  }
  pub fn get(&self)->&i64{
    &self.data
  }
}

impl Display for JrI64{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}",self.get())
  }
}

impl From<JrI64> for i64{
  fn from(data: JrI64) -> Self {
    *data.get()
  }
}

impl From<&JrI64> for i64{
  fn from(data: &JrI64) -> Self {
    *data.get()
  }
}

impl JrType for JrI64{
  fn get_bytes(&mut self, _:u8)->Vec<u8>{
    let mut data:Vec<u8> = Vec::new();
    let slice_i64: &[i64] = &*vec![self.data];
    for &n in slice_i64 {
      let _ = data.write_i64::<BigEndian>(n);
    }
    data
  }
}

#[derive(Clone)]
pub struct JrString{
  data:String
}

impl JrString{
  pub fn new(data:String)->Self{
    JrString{
      data
    }
  }
  pub fn get(&self)->&String{
    &self.data
  }
}

impl Display for JrString{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}",self.get())
  }
}

impl From<JrString> for String{
  fn from(data: JrString) -> Self {
    data.get().clone()
  }
}

impl From<&JrString> for String{
  fn from(data: &JrString) -> Self {
    data.get().clone()
  }
}

impl JrType for JrString{
  fn get_bytes(&mut self, _:u8)->Vec<u8>{
    self.data.clone().into_bytes()
  }
}

#[derive(Clone)]
pub struct JrCollection{
  data:Vec<JrDocument>
}



impl JrCollection{
  pub fn new()->Self{
    let data:Vec<JrDocument> = Vec::new();
    JrCollection{
      data
    }
  }

  pub fn add(&mut self,doc:JrDocument){
    self.data.push(doc);
  }

  pub fn get(&self, pos:usize)->&JrDocument{
    &self.data[pos]
  }

  pub fn len(&self)->usize{
    self.data.len()
  }

  pub fn print(&self, depth:u8){
    let space = "  ".repeat(depth as usize);
    for elem in self.data.iter() {
      if let Ok(s) = elem.get_value("_id") {
        let id:String = s;
        println!("{}_id: {}",space,id);
      }
      elem.print(depth+1);
    }

    println!("");
  }
}

impl JrType for JrCollection{
  fn get_bytes(&mut self, depth:u8)->Vec<u8>{
    let mut data = Vec::new();
    for (i, jr_doc) in self.data.iter_mut().enumerate() {
      let mut content_bytes = jr_doc.get_bytes(depth+1);
      let mut header = JrCollection::new_attr_header(2, depth+1,content_bytes.len() as u32, i.to_string());
      header.append(&mut content_bytes);
      data.append(&mut header);
    }
    data
  }
}

#[derive(Clone)]
pub struct JrDocument{
    data:BTreeMap<String, JrAny>,
}

impl JrDocument{
  pub fn new() -> JrDocument{
    let data:BTreeMap<String, JrAny> = BTreeMap::new();
    JrDocument{
      data
    }
  }

  pub fn get_content_bytes<T:JrType>(s:&mut T, content_type:u8, depth:u8, key:String)->Vec<u8>{
    let mut content_bytes = s.get_bytes(depth);
    let mut header = JrDocument::new_attr_header(content_type, depth, content_bytes.len() as u32, key);
    header.append(&mut content_bytes);
    header
  }

  pub fn print(&self, depth:u8){
    let space = "  ".repeat(depth as usize);
    for elem in self.data.iter() {
      let key = elem.0;
      if !key.eq("_id") {
        if let JrAny::JrI64(s) = elem.1{
          println!("{}{}: {}",space,key,s.get());
        }else if let JrAny::JrString(s) = elem.1{
          println!("{}{}: {}",space,key,s.get());
        }
      }
    }
    println!("");
  }

  pub fn loop_key<F>(&mut self, f:&mut F)
  where F:FnMut(&str, &mut JrAny)
  {
    for elem in self.data.iter_mut() {
      f(&elem.0, elem.1);
    }
  }
} 

impl JrType for JrDocument{
  fn get_bytes(&mut self, depth:u8)->Vec<u8>{
    let mut data = Vec::new();
    for elem in self.data.iter_mut() {
      let content = elem.1;
      let mut content_bytes = if let JrAny::JrI64(s) = content {

        JrDocument::get_content_bytes(s, 3, depth+1, String::from(elem.0))

      } else if let JrAny::JrString(s) = content {

        JrDocument::get_content_bytes(s, 2, depth+1, String::from(elem.0))

      } else if let JrAny::JrCollection(s) = content{

        JrDocument::get_content_bytes(s, 1, depth+1, String::from(elem.0))

      } else if let JrAny::JrDocument(s) = content{

        JrDocument::get_content_bytes(s, 0, depth+1, String::from(elem.0))

      } else {
        Vec::new()
      };

      data.append(&mut content_bytes);
      
    }
    data
  }
}

impl AddGet<JrDocument> for JrDocument{
  fn add(&mut self, key:&str, item: JrDocument){
    self.data.insert(key.to_string(), JrAny::JrDocument(item));
  }
  fn get(&self, key:&str)->Result<&JrDocument, &str>{
    if let None = self.data.get(key){
      Err("Key not found")
    }else if let JrAny::JrDocument(s) = self.data.get(key).unwrap(){
      Ok(s)
    }else{
      Err("Not a JrDocument")
    }   
  }
}

impl AddGet<JrCollection> for JrDocument{
  fn add(&mut self, key:&str, item: JrCollection){
    self.data.insert(key.to_string(), JrAny::JrCollection(item));
  }

  fn get(&self, key:&str)->Result<&JrCollection, &str>{
    if let None = self.data.get(key){
      Err("Key not found")
    }else if let JrAny::JrCollection(s) = self.data.get(key).unwrap(){
      Ok(s)
    }else{
      Err("Not a JrCollection")
    }   
  }

}

impl AddGet<JrString> for JrDocument{
  fn add(&mut self, key:&str, item: JrString){
    self.data.insert(key.to_string(), JrAny::JrString(item));
  }
  fn get(&self, key:&str)->Result<&JrString, &str>{
    if let None = self.data.get(key){
      Err("Key not found")
    }else if let JrAny::JrString(s) = self.data.get(key).unwrap(){
      Ok(s)
    }else{
      Err("Not a JrString")
    }   
  }
}

impl AddGet<JrI64> for JrDocument{
  fn add(&mut self, key:&str, item: JrI64){
    self.data.insert(key.to_string(), JrAny::JrI64(item));
  }
  fn get(&self, key:&str)->Result<&JrI64, &str>{
    if let None = self.data.get(key){
      Err("Key not found")
    }else if let JrAny::JrI64(s) = self.data.get(key).unwrap(){
      Ok(s)
    }else{
      Err("Not a JrString")
    }   
  }
}

impl AddGetValue<String> for JrDocument{
  fn add_value(&mut self, key:&str, item: String){
    let data = JrString::new(item);
    self.data.insert(key.to_string(), JrAny::JrString(data));
  }

  fn get_value(&self, key:&str)->Result<String, &str>{
    if let None = self.data.get(key){
      Err("Key not found")
    }else if let JrAny::JrString(s) = self.data.get(key).unwrap(){
      Ok(s.get().clone())
    }else{
      Err("Not a String")
    }   
  }

  fn get_value_from_db(&mut self, db:&mut Database, header:&mut HeaderDetail)->String{
    let data = match String::from_utf8(db.get_bytes_content(header)){
      Ok(d)=>d,
      Err(_)=>String::from(""),
    };
    self.add_value(&header.key, data.clone());
    data
  }
}

impl AddGetValue<i64> for JrDocument{
  fn add_value(&mut self, key:&str, item: i64){
    let data = JrI64::new(item);
    self.data.insert(key.to_string(), JrAny::JrI64(data));
  }

  fn get_value(&self, key:&str)->Result<i64, &str>{
    if let None = self.data.get(key){
      Err("Key not found")
    }else if let JrAny::JrI64(s) = self.data.get(key).unwrap(){
      Ok( *s.get() )
    }else{
      Err("Not a i64")
    }   
  }

  fn get_value_from_db(&mut self, db:&mut Database, header:&mut HeaderDetail)->i64{
    let data = db.get_bytes_content(header);

    let mut arr:[u8;8] = [0;8];
    for (i,item) in data.iter().enumerate() {
      arr[i] = *item;
    }
    
    self.add_value(&header.key, i64::from_be_bytes(arr));
    i64::from_be_bytes(arr)
  }
}