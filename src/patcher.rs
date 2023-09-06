use std::env::current_exe;
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
    key[..start.unwrap()].to_string()
}

fn set_property_by_path(
    json: &mut Value,
    path: &str,
    value: &mut Value,
    operation: Operation,
    force: bool,
) -> Result<(), &'static str> {
    let mut current = json;
    let mut keys = path.split('.');
    let mut current_key: String = String::new();
    let mut current_index: usize = 0;
    let mut enum_keys = keys.clone().enumerate();
    let mut is_array = false;
    let mut is_last_key = false;
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

                current_key = get_key(key);
                is_array = key.contains('[');
                is_last_key = enum_keys.clone().next().is_none();

                if let Value::Object(obj) = current {
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

                    if is_last_key && obj[current_key.as_str()] == json!({}) {
                        obj.insert(current_key.to_string(), value.clone());
                        return Ok(());
                    }

                    current = &mut obj[current_key.as_str()];
                    println!("{:}", current);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use serde_json::Value::Null;
    use crate::patcher::{Operation, set_property_by_path};

    #[test]
    fn test_patch_add_list() {
        let mut base_json = json!({});
        let path = "$.list";
        let mut value = json!([1,2,3]);

        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [1,2,3]
        }))
    }

    #[test]
    fn test_patch_add_element_to_list() {
        let mut base_json = json!({});
        let path = "$.list[0]";
        let mut value = json!(1);

        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [1]
        }));

        let path = "$.list[0]";
        let mut value = json!(2);
        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Change, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2]
        }));


        let path = "$.list[1]";
        let mut value = json!(3);
        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2,3]
        }));

        let path = "$.list[1]";
        let mut value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2]
        }));

        let path = "$.list[0]";
        let mut value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": []
        }));

        let path = "$.list";
        let mut value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &mut value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({}));
    }
}
