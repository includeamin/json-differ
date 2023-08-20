use serde_json::{Value, json};

#[derive(Debug)]
enum Operation {
    Add,
    Change,
    Delete,
}

fn get_key(key: &str) -> String {
    if !key.contains('[') {
        return key.to_string();
    }

    let start = key.find('[');
    let end = key.find(']');
    key[..start.unwrap()].to_string()
}

fn set_property_by_path(json: &mut Value, path: &str, value: &mut Value, operation: Operation, force: bool) -> Result<(), &'static str> {
    let mut current = json;
    let mut keys = path.split('.');
    let mut current_key: String = String::new();
    let mut current_index: usize = 0;
    let mut enum_keys = keys.clone().enumerate();

    loop {
        let next_key = enum_keys.next();
        match next_key {
            None => {
                match current {
                    Value::Null => {
                        println!("null: {:?}", current);
                    }
                    Value::Bool(_) => {
                        println!("bool: {:?}", current);
                    }
                    Value::Number(_) => {
                        println!("number: {:?}", current);
                    }
                    Value::String(_) => {
                        println!("string: {:?}", current);
                    }
                    Value::Array(arr) => {
                        println!("array: {:?}", arr);
                        if arr.len() <= current_index {
                            return Err("index out of bounds");
                        }

                        match operation {
                            Operation::Add => {
                                if arr.len() < current_index {
                                    return Err("index out of bounds");
                                }
                                let value_at_index = arr.get(current_index);
                                match value_at_index {
                                    None => {
                                        arr.insert(current_index, value.clone());
                                    }
                                    Some(_) => {
                                        if force {
                                            arr.insert(current_index, value.clone());
                                        } else {
                                            return Err("index already has value");
                                        }
                                    }
                                }
                            }
                            Operation::Change => {
                                if arr.len() < current_index {
                                    return Err("index out of bounds");
                                }
                                arr[current_index] = value.clone();
                            }
                            Operation::Delete => {
                                if arr.len() < current_index {
                                    return Err("index out of bounds");
                                }
                                arr.remove(current_index);
                            }
                        }
                    }

                    _ => {
                        return Ok(());
                    }
                }

                return Ok(());
            }
            Some((_, key)) => {
                if key == "$" {
                    continue;
                }

                current_key = get_key(key);
                let is_array = key.contains('[');
                let is_last_key = enum_keys.clone().next().is_none();

                if let Value::Object(obj) = current {
                    if is_array {
                        let start = key.find('[');
                        let end = key.find(']');
                        let index: usize = key[start.unwrap() + 1..end.unwrap()]
                            .parse()
                            .unwrap();
                        current_index = index;
                    }

                    if !obj.contains_key(current_key.as_str()) && !is_last_key {
                        println!("imk");
                        obj.insert(current_key.to_string(), json!({}));
                    }

                    if !obj.contains_key(current_key.as_str()) && is_last_key {
                        println!("last key");
                        obj.insert(current_key.to_string(), value.clone());
                    }

                    current = &mut obj[current_key.as_str()];
                    println!("current key: {:?}", current_key);
                    println!("current: {:?}", current);

                    //
                    // if is_last_key & !is_array {
                    //     println!("inja");
                    //     match operation {
                    //         Operation::Add => {
                    //             if obj.contains_key(current_key.as_str()) {
                    //                 if force {
                    //                     obj[current_key.as_str()] = value.clone();
                    //                 } else {
                    //                     return Err("key already exists");
                    //                 }
                    //             } else {
                    //                 obj.insert(current_key.to_string(), value.clone());
                    //             }
                    //         }
                    //         Operation::Change => {
                    //             if obj.contains_key(current_key.as_str()) {
                    //                 obj[current_key.as_str()] = value.clone();
                    //             } else {
                    //                 return Err("key does not exist");
                    //             }
                    //         }
                    //         Operation::Delete => {
                    //             if obj.contains_key(current_key.as_str()) {
                    //                 obj.remove(current_key.as_str());
                    //             } else {
                    //                 return Err("key does not exist");
                    //             }
                    //         }
                    //     }
                    //     return Ok(());
                    // }
                }
            }
        }
    }
}


fn main() {
    let mut json = json!({
        "foo": {
            "age": 11,
            "list": [
                {
                    "list": [
                        1, 2, 3
                    ]
                }
            ]
        }
    });

    let test_path = "$.foo.list";
    let mut value = json!({
        "name": "bar",
        "address": "1234"
    });

    let result = set_property_by_path(&mut json, test_path, &mut value, Operation::Add, false);
    match result {
        Ok(_) => {
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        Err(err) => {
            println!("error: {}", err);
        }
    }
}
