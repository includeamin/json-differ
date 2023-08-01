use serde_json::Value;
use serde_json_path::JsonPath;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Operation {
    Add,
    Change,
    Delete,
}

#[derive(Debug, PartialEq)]
struct Delta {
    operation: Operation,
    path: String,
    old_value: Value,
    new_value: Value,
}

struct Diff {
    deltas: Vec<Delta>,
    paths: Vec<String>,
    a: Value,
    b: Value,
}

impl Default for Diff {
    fn default() -> Self {
        Diff {
            deltas: Vec::new(),
            paths: Vec::new(),
            a: Value::Null,
            b: Value::Null,
        }
    }
}

impl Diff {
    pub fn new_from_json_values(a: Value, b: Value) -> Diff {
        Diff {
            a,
            b,
            deltas: Vec::new(),
            paths: Vec::new(),
        }
    }

    /// Returns the deltas between the two values
    fn get_deltas(&self) -> &Vec<Delta> {
        &self.deltas
    }

    /// Returns the paths that were changed
    fn get_paths(&self) -> &Vec<String> {
        &self.paths
    }

    /// Returns true if there are any changes between the two values
    fn has_changes(&self) -> bool {
        !self.deltas.is_empty()
    }

    /// Returns true if the two values are equal
    fn diff(&mut self) -> &Self {
        let mut paths: Vec<String> = Vec::new();
        let mut current_path: Vec<String> = Vec::new();
        let mut seen: HashMap<String, bool> = HashMap::new();

        diff(
            &self.a,
            &self.b,
            &mut current_path,
            &mut paths,
            &mut self.deltas,
            &mut seen,
            false,
        );

        diff(
            &self.b,
            &self.a,
            &mut current_path,
            &mut paths,
            &mut self.deltas,
            &mut seen,
            true,
        );

        self
    }
}

fn diff(
    left: &Value,
    right: &Value,
    current_path: &mut Vec<String>,
    paths: &mut Vec<String>,
    deltas: &mut Vec<Delta>,
    seen: &mut HashMap<String, bool>,
    reverse: bool,
) {
    match left {
        Value::Object(map) => {
            for (key, value) in map.iter() {
                if current_path.is_empty() {
                    current_path.push(format!("$.{}", key));
                } else {
                    current_path.push(key.to_string());
                }
                diff(value, right, current_path, paths, deltas, seen, reverse);
                current_path.pop();
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                match current_path.len() {
                    0 => current_path.push(format!("$[{}]", index)),
                    _ => current_path.push(format!("[{}]", index)),
                }
                diff(value, right, current_path, paths, deltas, seen, reverse);
                current_path.pop();
            }
        }
        _ => {
            let mut path = String::new();
            for (index, value) in current_path.iter().enumerate() {
                if value.contains('[') {
                    path.push_str(value);
                    continue;
                }
                if index == 0 {
                    path.push_str(value);
                    continue;
                }
                path.push_str(format!(".{}", value).as_str());
            }

            if seen.contains_key(path.as_str()) {
                return;
            }

            let parsed_path = JsonPath::parse(path.as_str()).unwrap();
            let left_value = parsed_path.query(right).first();
            match left_value {
                // the value exists in both, so we need to check if it's the same
                Some(value) => {
                    if left != value {
                        deltas.push(Delta {
                            operation: Operation::Change,
                            path: path.clone(),
                            old_value: left.clone(),
                            new_value: value.clone(),
                        });
                    }
                }
                // the value doesn't exist in b, so we need to delete it
                None => {
                    if reverse {
                        deltas.push(Delta {
                            operation: Operation::Add,
                            path: path.clone(),
                            old_value: Value::Null,
                            new_value: left.clone(),
                        });
                    } else {
                        deltas.push(Delta {
                            operation: Operation::Delete,
                            path: path.clone(),
                            old_value: left.clone(),
                            new_value: Value::Null,
                        });
                    }
                }
            }

            seen.insert(path.clone(), true);
            paths.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn diff_from_serde_values_success() {
        // given
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/testdata/small_json_1.json");
        let data = fs::read_to_string(d).expect("Unable to read file");
        let json_a = serde_json::from_str(&data).expect("Unable to parse");

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/testdata/small_json_2.json");
        let data = fs::read_to_string(d).expect("Unable to read file");
        let json_b = serde_json::from_str(&data).expect("Unable to parse");

        let mut differ = Diff::new_from_json_values(json_a, json_b);
        let deltas = differ.diff().get_deltas();

        // then
        assert_eq!(deltas.len(), 4, "Expected 4 deltas, got {}", deltas.len());

        assert_eq!(deltas[0].operation, Operation::Change);
    }

    #[test]
    fn diff_from_json_value() {
        let a = serde_json::from_str(
            r#"
         {
           "username": "admin",
           "password": "admin",
           "email": "foo@bar.com",
           "name": "Foo Bar",
           "address": "Dhaka",
           "roles": [
             "admin"
           ],
           "status": "active",
           "created_at": "2019-12-12T12:12:12.000Z",
           "nested": {
             "foo": "bar"
           }
         }
        "#,
        )
        .unwrap();

        let b = serde_json::from_str(
            r#"
         {
           "username": "admin",
           "password": "admin",
           "email": "foo@bar.com",
           "name": "Foo Bar",
           "address": "Dhaka",
           "phone": "123456789",
           "roles": [
             "admin"
           ],
           "status": "active",
           "created_at": "2019-12-12T12:12:13.000Z",
           "nested": {
             "foo": "bar"
           }
         }
        "#,
        )
        .unwrap();

        let mut differ = Diff::new_from_json_values(a, b);
        let deltas = differ.diff().get_deltas();

        println!("deltas: {:?}", deltas);
        assert_eq!(deltas.len(), 2, "Expected 1 deltas, got {}", deltas.len());
    }
}
