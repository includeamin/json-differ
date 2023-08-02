use serde_json::Value;
use serde_json_path::JsonPath;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, PartialEq)]
pub enum Operation {
    Add,
    Change,
    Delete,
}

#[derive(Debug, PartialEq)]
pub struct Delta {
    operation: Operation,
    path: String,
    old_value: Value,
    new_value: Value,
}

#[derive(Debug, PartialEq)]
pub struct Diff {
    deltas: Vec<Delta>,
    paths: Vec<String>,
    left: Value,
    right: Value,
}

impl Default for Diff {
    fn default() -> Self {
        Diff {
            deltas: Vec::new(),
            paths: Vec::new(),
            left: Value::Null,
            right: Value::Null,
        }
    }
}

impl Diff {
    pub fn new_from_json_values(a: Value, b: Value) -> Diff {
        Diff {
            left: a,
            right: b,
            deltas: Vec::new(),
            paths: Vec::new(),
        }
    }

    /// Returns the deltas between the two values
    pub fn get_deltas(&self) -> &Vec<Delta> {
        &self.deltas
    }

    /// Returns the delta for the given path
    pub fn get_delta_by_path(&self, path: &str) -> Option<&Delta> {
        self.deltas.iter().find(|delta| delta.path == path)
    }

    /// Returns the paths that were changed
    pub fn get_paths(&self) -> &Vec<String> {
        &self.paths
    }

    /// Returns true if the given path has changed
    pub fn has_path_changed(&self, path: &str, operation: Operation) -> bool {
        self.deltas.iter().any(|delta| delta.path == path && delta.operation == operation)
    }

    /// Returns true if there are any changes between the two values
    pub fn has_changes(&self) -> bool {
        !self.deltas.is_empty()
    }

    /// Returns true if the two values are equal
    pub fn diff(&mut self) -> &Self {
        let mut paths: Vec<String> = Vec::new();
        let mut seen: HashMap<String, bool> = HashMap::new();
        let mut deltas: Vec<Delta> = Vec::new();

        self.do_diff(
            &self.left,
            &self.right,
            &mut paths,
            &mut deltas,
            &mut seen,
            false,
        );

        self.do_diff(
            &self.right,
            &self.left,
            &mut paths,
            &mut deltas,
            &mut seen,
            true,
        );

        self.deltas = deltas;

        self
    }


    #[allow(clippy::too_many_arguments)]
    fn do_diff(
        &self,
        left: &Value,
        right: &Value,
        paths:  &mut Vec<String>,
        deltas: &mut Vec<Delta>,
        seen: &mut HashMap<String, bool>,
        reverse: bool,
    ) {
        let current_path = Vec::new();
        let mut stack: VecDeque<(&Value, &Value, Vec<String>)> = VecDeque::new();
        stack.push_back((left, right, current_path));

        while let Some((left, right, path)) = stack.pop_back() {
            match left {
                Value::Object(map) => {
                    for (key, value) in map.iter() {
                        let mut new_path = path.clone();
                        if path.is_empty() {
                            new_path.push(format!("$.{}", key));
                        } else {
                            new_path.push(key.to_string());
                        }
                        stack.push_back((value, right, new_path));
                    }
                }
                Value::Array(array) => {
                    for (index, value) in array.iter().enumerate() {
                        let mut new_path = path.clone();
                        match path.len() {
                            0 => new_path.push(format!("$[{}]", index)),
                            _ => new_path.push(format!("[{}]", index)),
                        }
                        stack.push_back((value, right, new_path));
                    }
                }
                _ => {
                    let mut path_str = String::new();
                    for (index, value) in path.iter().enumerate() {
                        if value.contains('[') || index == 0 {
                            path_str.push_str(value);
                        } else {
                            path_str.push_str(format!(".{}", value).as_str());
                        }
                    }

                    if seen.contains_key(path_str.as_str()) {
                        continue;
                    }

                    let parsed_path = JsonPath::parse(path_str.as_str()).unwrap();
                    let left_value = parsed_path.query(right).first();
                    match left_value {
                        // the value exists in both, so we need to check if it's the same
                        Some(value) => {
                            if left != value {
                                deltas.push(Delta {
                                    operation: Operation::Change,
                                    path: path_str.clone(),
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
                                    path: path_str.clone(),
                                    old_value: Value::Null,
                                    new_value: left.clone(),
                                });
                            } else {
                                deltas.push(Delta {
                                    operation: Operation::Delete,
                                    path: path_str.clone(),
                                    old_value: left.clone(),
                                    new_value: Value::Null,
                                });
                            }
                        }
                    }

                    seen.insert(path_str.clone(), true);
                    paths.push(path_str);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
             "foo": "bar",
             "nested2": {
               "foo": "bar",
                "nested3": {
                  "foo": "bar"
                }
             }
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
             "foo": "bar",
                "nested2": {
                "foo": "bar",
                    "nested3": {
                    "foo": "bar2"
                    }
                }
           }
         }
        "#,
        )
            .unwrap();

        let mut differ = Diff::new_from_json_values(a, b);
        let differ = differ.diff();

        assert_eq!(differ.get_deltas().len(), 3, "Expected 1 deltas, got {}", differ.get_deltas().len());
        assert!(differ.has_changes());
        assert!(differ.has_path_changed("$.phone", Operation::Add));
        assert!(differ.has_path_changed("$.created_at", Operation::Change));

        let delta = differ.get_delta_by_path("$.phone").unwrap();
        assert_eq!(delta.operation, Operation::Add);
        assert_eq!(delta.path, "$.phone");
        assert_eq!(delta.old_value, Value::Null);
        assert_eq!(delta.new_value, Value::String("123456789".to_string()));

        let delta = differ.get_delta_by_path("$.created_at").unwrap();
        assert_eq!(delta.operation, Operation::Change);
        assert_eq!(delta.path, "$.created_at");
        assert_eq!(delta.old_value, Value::String("2019-12-12T12:12:12.000Z".to_string()));
        assert_eq!(delta.new_value, Value::String("2019-12-12T12:12:13.000Z".to_string()));

        let delta = differ.get_delta_by_path("$.nested.nested2.nested3.foo").unwrap();
        assert_eq!(delta.operation, Operation::Change);
        assert_eq!(delta.path, "$.nested.nested2.nested3.foo");
        assert_eq!(delta.old_value, Value::String("bar".to_string()));
        assert_eq!(delta.new_value, Value::String("bar2".to_string()));
    }
}
