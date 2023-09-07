use serde_json::{Value, json};
use crate::delta::{Delta, Operation};

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


pub fn patch(base: Value, deltas: Vec<Delta>) -> Value {
    let base_value = &mut base.clone();

    for delta in deltas {
        set_property_by_path(base_value, delta.path.as_str(), &delta.new_value, delta.operation, false).unwrap()
    }

    base_value.clone()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use serde_json::Value::Null;
    use crate::delta::Delta;
    use crate::patcher::{Operation, patch, set_property_by_path};

    #[test]
    fn test_patch_add_list() {
        let mut base_json = json!({});
        let path = "$.list";
        let value = json!([1,2,3]);

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [1,2,3]
        }))
    }

    #[test]
    fn test_patch_add_element_to_list() {
        let mut base_json = json!({});
        let path = "$.list[0]";
        let value = json!(1);

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [1]
        }));

        let path = "$.list[0]";
        let value = json!(2);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2]
        }));


        let path = "$.list[1]";
        let value = json!(3);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2,3]
        }));

        let path = "$.list[1]";
        let value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2]
        }));

        let path = "$.list[0]";
        let value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": []
        }));

        let path = "$.list";
        let value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_number() {
        let mut base_json = json!({});
        let path = "$.age";
        let value = json!(1);

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();
        assert_eq!(base_json, json!({"age": 1}));

        let value = json!(2);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();
        assert_eq!(base_json, json!({"age": 2}));


        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();
        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_string() {
        let mut base_json = json!({});
        let path = "$.first_name";
        let value = json!("first name");

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();
        assert_eq!(base_json, json!({"first_name": "first name"}));

        let value = json!("changed name");
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();
        assert_eq!(base_json, json!({"first_name": "changed name"}));


        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();
        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_nested_json() {
        let mut base_json = json!({});
        let path = "$.gdpr.first_name";
        let value = json!("first name");

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();
        assert_eq!(base_json, json!({"gdpr": {"first_name": "first name"}}));

        let value = json!("changed name");
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();
        assert_eq!(base_json, json!({"gdpr": {"first_name": "changed name"}}));

        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();
        assert_eq!(base_json, json!({"gdpr":{}}));
    }

    #[test]
    fn test_path() {
        let deltas = vec![
            Delta {
                operation: Operation::Add,
                path: "$.age".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(1),
            },
            Delta {
                operation: Operation::Add,
                path: "$.personal_information.first_name".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!("first name"),
            },
            Delta {
                operation: Operation::Change,
                path: "$.age".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(20),
            },
            Delta {
                operation: Operation::Add,
                path: "$.tags".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(["test","test1"]),
            },
            Delta {
                operation: Operation::Change,
                path: "$.tags[1]".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!("test2"),
            },
        ];

        let patched = patch(json!({}), deltas);

        assert_eq!(patched, json!(
            {
                "age":20 ,
                "personal_information": {"first_name": "first name"},
                "tags":["test","test2"]
            }
        ))
    }
}
