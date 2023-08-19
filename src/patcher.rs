use serde_json::{Value, json, Map};
use crate::delta::Operation;


fn set_property_by_path(json: &mut Value, path: &str, value: &mut Value, operation: Operation, force: bool) -> Result<(), &'static str> {
    let mut current = json;
    let mut keys = path.split('.');
    let total_keys = keys.clone().count();
    let mut current_index: usize = 0;
    let mut current_key: String = String::new();


    for (index, key) in keys.enumerate() {
        println!("key: {}", key);
        if key == "$" {
            continue;
        }

        let is_last_key = index == total_keys - 1;

        match current {
            Value::Object(obj) => {
                println!("object: {:?}", obj);
                current_key = key.clone().to_string();

                if current_key.contains('[') {
                    let start = current_key.find('[');
                    let end = current_key.find(']');
                    let index: usize = current_key[start.unwrap() + 1..end.unwrap()]
                        .parse()
                        .unwrap();
                    current_index = index;
                    current_key = current_key[..start.unwrap()].to_string();
                }

                if !obj.contains_key(current_key.as_str()) && !is_last_key {
                    obj.insert(current_key.to_string(), json!({}));
                }

                if is_last_key {
                    let tmp = &current_key.clone();
                    // println!("current_key: {}", current_key);
                    // obj[tmp] = value.clone();
                    match operation {
                        Operation::Add => {
                            if obj.contains_key(current_key.as_str()) {
                                if force {
                                    obj[tmp] = value.clone();
                                } else {
                                    return Err("key already exists");
                                }
                            } else {
                                obj.insert(current_key.to_string(), value.clone());
                            }
                        }
                        Operation::Change => {
                            if obj.contains_key(current_key.as_str()) {
                                obj[tmp] = value.clone();
                            } else {
                                return Err("key does not exist");
                            }
                        }
                        Operation::Delete => {
                            if obj.contains_key(current_key.as_str()) {
                                obj.remove(current_key.as_str());
                            } else {
                                return Err("key does not exist");
                            }
                        }
                    }
                    return Ok(());
                } else {
                    let tmp = &current_key.clone();
                    current = &mut obj[tmp];
                }
            }

            Value::Array(arr) => {
                println!("array: {:?}", arr);
                if arr.len() <= current_index {
                    return Err("index out of bounds");
                }

                current = &mut arr[current_index];
                let array = current[&current_key].as_array_mut().unwrap();

                if !is_last_key {
                    continue;
                }


                match operation {
                    Operation::Add => {
                        if array.len() < current_index {
                            return Err("index out of bounds");
                        }
                        let value_at_index = array.get(current_index);
                        match value_at_index {
                            None => {
                                array.insert(current_index, value.clone());
                            }
                            Some(_) => {
                                if force {
                                    array.insert(current_index, value.clone());
                                } else {
                                    return Err("index already has value");
                                }
                            }
                        }
                    }
                    Operation::Change => {
                        if array.len() < current_index {
                            return Err("index out of bounds");
                        }

                        let value_at_index = array.get(current_index);
                        match value_at_index {
                            None => {
                                if force {
                                    array.insert(current_index, value.clone());
                                } else {
                                    return Err("index already has value");
                                }
                            }
                            Some(_) => {
                                array[current_index] = value.clone();
                            }
                        }
                    }
                    Operation::Delete => {
                        if array.len() < current_index {
                            return Err("index out of bounds");
                        }
                        current[&current_key].as_array_mut().unwrap().remove(current_index);
                    }
                }
            }

            Value::Number(num) => {
                println!("number: {:?}", num);
                if is_last_key {
                    match operation {
                        Operation::Add => {
                            return Err("cannot add value to number");
                        }
                        Operation::Change => {
                            println!("here");
                            *current = value.clone();
                        }
                        Operation::Delete => {
                            return Err("cannot delete value of number");
                        }
                    }
                }
            }
            Value::Null => {}
            Value::Bool(_) => {}
            Value::String(_) => {}
        }
    }

    Ok(())
}
