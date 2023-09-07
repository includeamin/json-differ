use serde_json::{Value, json};
use crate::delta::{Delta, Operation};

fn get_key(key: &str) -> String {
    if !key.contains('[') {
        return key.to_string();
    }

    let start = key.find('[');
    key[..start.unwrap()].to_string()
}

pub(crate) fn set_property_by_path(
    json: &mut Value,
    path: &str,
    value: &Value,
    operation: Operation,
    force: bool,
) -> Result<(), &'static str> {
    let mut current = json;
    let keys = path.split('.');
    let mut current_index: usize = usize::default();
    let mut enum_keys = keys.clone().enumerate();
    loop {
        let next_key = enum_keys.next();
        match next_key {
            None => {
                match current {
                    Value::Array(arr) => {
                        match operation {
                            Operation::Add => {
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
                                if arr.len() <= current_index {
                                    return Err("index out of bounds");
                                }
                                arr[current_index] = value.clone();
                            }
                            Operation::Delete => {
                                if arr.len() <= current_index {
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

                let current_key = get_key(key);
                let is_array = key.contains('[');
                let is_last_key = enum_keys.clone().next().is_none();

                if let Value::Object(obj) = current {
                    if is_last_key && !is_array {
                        return match operation {
                            Operation::Add => {
                                obj.insert(current_key.to_string(), value.clone());
                                Ok(())
                            }
                            Operation::Change => {
                                obj.insert(current_key.to_string(), value.clone());
                                Ok(())
                            }
                            Operation::Delete => {
                                obj.remove(current_key.as_str());
                                Ok(())
                            }
                        };
                    }

                    if is_array {
                        let start = key.find('[');
                        let end = key.find(']');
                        let index: usize = key[start.unwrap() + 1..end.unwrap()]
                            .parse()
                            .unwrap();
                        current_index = index;
                    }
                    if is_array && !obj.contains_key(current_key.as_str()) {
                        obj.insert(current_key.to_string(), json!([]));
                    } else if !obj.contains_key(current_key.as_str()) {
                        obj.insert(current_key.to_string(), json!({}));
                    }


                    current = &mut obj[current_key.as_str()];
                }
            }
        }
    }
}


pub fn patch(base: Value, deltas: &Vec<Delta>) -> Value {
    let base_value = &mut base.clone();

    for delta in deltas {
        set_property_by_path(
            base_value,
            delta.path.as_str(),
            &delta.new_value,
            delta.operation.clone(),
            false).unwrap()
    }

    base_value.clone()
}
