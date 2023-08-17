use serde_json::{json, Value};
use crate::delta::Operation;
use serde_json::Value::String;

fn set_property_by_path(json: &mut Value, path: &str, value: &mut Value, operation: Operation, force: bool) {
    let mut current = json;
    let mut keys = path.split('.');
    let total_keys = keys.clone().count();
    let mut current_index: usize = 0;


    for (index, key) in keys.enumerate() {
        println!("key: {}", key);
        if key == "$" {
            continue;
        }

        let is_last_key = index == total_keys - 1;

        match current {
            Value::Object(obj) => {
                println!("object: {:?}", obj);
                let mut current_key = key.clone().to_string();

                if current_key.contains('[') {
                    let start = current_key.find('[');
                    let end = current_key.find(']');
                    let index: usize = current_key[start.unwrap() + 1..end.unwrap()]
                        .parse()
                        .unwrap();
                    current_index = index;
                    current_key = current_key[..start.unwrap()].to_string();
                }

                if !obj.contains_key(current_key.as_str()) {
                    obj.insert(current_key.to_string(), json!({}));
                }

                let tmp = &current_key.clone();
                current = &mut obj[tmp];
                if is_last_key {
                    process_op(current, key, value, &operation, force);
                }
            }

            Value::Null => {}
            Value::Bool(_) => {}
            Value::Number(_) => {}
            String(str_val) => {
                println!("string: {}", str_val);
            }
            Value::Array(arr) => {
                println!("array: {:?}", arr);

                if arr.len() <= current_index {
                    eprintln!("index out of bounds");
                    return;
                }

                current = &mut arr[current_index];
                if is_last_key {
                    process_op(current, key, value, &operation, force);
                }
            }
        }
    }
}

fn process_op(value: &mut Value, key: &str, set_to: &Value, operation: &Operation, force: bool) {
    println!("process_op");
    match value {
        Value::Object(obj) => {
            match operation {
                Operation::Add => {
                    let has_attr = obj.contains_key(key);
                    if has_attr && !force {
                        eprintln!("attribute already exists");
                    } else {
                        obj.insert(key.to_string(), set_to.clone());
                    }
                }
                Operation::Change => {
                    let has_attr = obj.contains_key(key);
                    if !has_attr {
                        eprintln!("attribute does not exist");
                    } else {
                        obj.insert(key.to_string(), set_to.clone());
                    }
                }
                Operation::Delete => {
                    let has_attr = obj.contains_key(key);
                    if !has_attr {
                        eprintln!("attribute does not exist");
                    } else {
                        obj.remove(key);
                    }
                }
            }
        }
        Value::Null => {
            println!("null");
        }
        Value::Bool(bool_val) => {
            println!("bool: {}", bool_val);
        }
        Value::Number(number_val) => {
            println!("number: {}", number_val);
        }
        String(str_val) => {
            println!("string: {}", str_val);
        }
        Value::Array(arr) => {
            println!("array: {:?}", arr);
        }
    }
}
//
// pub struct Patcher {
//     pub base: Value,
// }
//
// impl Patcher {
//     pub fn new(base: Value) -> Self {
//         if !base.is_object() {
//             panic!("Patcher only works with objects");
//         }
//         Patcher {
//             base,
//         }
//     }
//
//     fn set_path_property_by_path(&mut self, path: &str, value: Value, operation: Operation, force: bool) {
//         let mut current = &mut self.base;
//         let mut keys = path.split('.');
//
//         while let Some(key) = keys.next() {
//             if key == "$" {
//                 continue;
//             }
//
//             match current {
//                 Value::Object(obj) => {
//                     if let Some(val) = keys.next() {
//                         if obj.contains_key(key) {
//                             current = obj.get_mut(key).unwrap();
//                         } else {
//                             let new_obj = json!({});
//                             obj.insert(key.to_string(), new_obj);
//                             current = obj.get_mut(key).unwrap();
//                         }
//                         return;
//                     }
//
//
//                     let current_key = key.clone();
//                     if current_key.contains("[") {
//                         // TODO: move this block to a helper function ---
//                         let start = current_key.find("[");
//                         let end = current_key.find("]");
//                         let index: usize = current_key[start.unwrap() + 1..end.unwrap()].parse().unwrap();
//                         let actual_key = current_key[..start.unwrap()].to_string();
//                         // TODO: -----
//
//                         match obj.get(actual_key.as_str()).as_mut() {
//                             None => {
//                                 obj.insert(actual_key.to_string(), json!([value]));
//                                 return;
//                             }
//                             Some(array) => {
//                                 match array.clone() {
//                                     Value::Array(mut array_value) => {
//                                         match operation {
//                                             Operation::Add => {
//                                                 if array_value.get(index).is_some() {
//                                                     if !force {
//                                                         eprintln!("Conflict: Key already exists");
//                                                         return;
//                                                     }
//
//                                                     array_value[index] = value.clone();
//                                                     return;
//                                                 }
//
//                                                 array_value.resize(index + 1, value.clone());
//                                             }
//                                             Operation::Change => {
//                                                 if array_value.get(index).is_some() {
//                                                     array_value[index] = value.clone();
//                                                     return;
//                                                 }
//                                                 eprintln!("Conflict: Key does not exist");
//                                                 return;
//                                             }
//                                             Operation::Delete => {
//                                                 if array_value.get(index).is_some() {
//                                                     array_value.remove(index);
//                                                     return;
//                                                 }
//                                                 eprintln!("Conflict: Key does not exist");
//                                                 return;
//                                             }
//                                         }
//                                     }
//                                     _ => {
//                                         eprintln!("Invalid JSON path");
//                                         return;
//                                     }
//                                 }
//                             }
//                         }
//                     }
//
//                     match operation {
//                         Operation::Add => {
//                             if obj.contains_key(key) {
//                                 if !force {
//                                     eprintln!("Conflict: Key already exists");
//                                     return;
//                                 }
//
//                                 obj[key] = value.clone();
//                                 return;
//                             }
//
//                             obj.insert(key.to_string(), value.clone());
//                         }
//                         Operation::Change => {
//                             if obj.contains_key(key) {
//                                 obj[key] = value.clone();
//                                 return;
//                             }
//
//                             eprintln!("Conflict: Key does not exist");
//                             return;
//                         }
//                         Operation::Delete => {
//                             if obj.contains_key(key) {
//                                 obj.remove(key);
//                                 return;
//                             }
//
//                             eprintln!("Conflict: Key does not exist");
//                             return;
//                         }
//                     }
//                 }
//
//                 _ => {
//                     eprintln!("Invalid JSON path");
//                     return;
//                 }
//             }
//         }
//     }
// }
