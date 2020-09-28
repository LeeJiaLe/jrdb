use std::collections::HashMap;
use byteorder::{WriteBytesExt, BigEndian};

enum JrAny{
  JrCollection(JrCollection),
  JrDocument(JrDocument),
  JrString(JrString),
}

pub trait AddGet<T>
where T:JrType
{
  fn add(&mut self, key:&str, item: T);
  fn get(&self, key:&str)->Result<&T, &str>;
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

// struct TypeCheck{}

// impl TypeCheck{
//     pub fn type_of<T>(_: T) -> &'static str {
//         type_name::<T>()
//     }
// }

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

impl JrType for JrString{
  fn get_bytes(&mut self, _:u8)->Vec<u8>{
    format!("{}",self.data).into_bytes()
  }
}

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
}

impl JrType for JrCollection{
  fn get_bytes(&mut self, depth:u8)->Vec<u8>{
    let mut data = Vec::new();
    for (i, jr_doc) in self.data.iter_mut().enumerate() {
      let mut content_bytes = jr_doc.get_bytes(depth+1);
      let mut header = JrCollection::new_attr_header(2, depth+1,content_bytes.len() as u32, format!("{}",i));
      header.append(&mut content_bytes);
      data.append(&mut header);
    }
    data
  }
}

pub struct JrDocument{
    data:HashMap<String, JrAny>,
}

impl JrDocument{
  pub fn new() -> JrDocument{
    let data:HashMap<String, JrAny> = HashMap::new();
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
} 

impl JrType for JrDocument{
  fn get_bytes(&mut self, depth:u8)->Vec<u8>{
    let mut data = Vec::new();
    for elem in self.data.iter_mut() {
      let content = elem.1;
      let mut content_bytes = if let JrAny::JrString(s) = content {

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
      Err("Not a String")
    }   
  }
}

