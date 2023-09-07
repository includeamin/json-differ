use std::collections::{HashMap, VecDeque};
use serde_json::Value;
use serde_json_path::JsonPath;
use crate::delta::{Delta, Operation};
use crate::utils::calculate_hash;

#[derive(Debug, PartialEq)]
pub struct Differ {
    deltas: Vec<Delta>,
    left: Value,
    right: Value,
}


impl Default for Differ {
    fn default() -> Self {
        Differ {
            deltas: Vec::new(),
            left: Value::Null,
            right: Value::Null,
        }
    }
}

impl Differ {
    pub fn new_from_json_values(a: Value, b: Value) -> Differ {
        Differ {
            left: a,
            right: b,
            deltas: Vec::new(),
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

    /// Returns true if the given path has changed
    pub fn has_path_changed(&self, path: &str, operation: Operation) -> bool {
        self.deltas
            .iter()
            .any(|delta| delta.path == path && delta.operation == operation)
    }

    /// Returns true if there are any changes between the two values
    pub fn has_changes(&self) -> bool {
        !self.deltas.is_empty()
    }

    /// Returns true if the two values are equal
    pub fn diff(&mut self) -> &Self {
        let mut seen: HashMap<String, bool> = HashMap::new();
        let mut deltas: Vec<Delta> = Vec::new();

        self.do_diff(&self.left, &self.right, &mut deltas, &mut seen, false);

        self.do_diff(&self.right, &self.left, &mut deltas, &mut seen, true);

        self.deltas = deltas;

        self
    }

    fn do_diff(
        &self,
        left: &Value,
        right: &Value,
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
                        Some(value) => {
                            if left != value {
                                let mut delta = Delta {
                                    operation: Operation::Change,
                                    path: path_str.clone(),
                                    old_value: left.clone(),
                                    new_value: value.clone(),
                                    hash: String::default(),
                                };

                                delta.hash = calculate_hash(&delta).to_string();
                                deltas.push(delta);
                            }
                        }
                        None => {
                            if reverse {
                                let mut delta = Delta {
                                    operation: Operation::Add,
                                    path: path_str.clone(),
                                    old_value: Value::Null,
                                    new_value: left.clone(),
                                    hash: String::default(),
                                };
                                delta.hash = calculate_hash(&delta).to_string();
                                deltas.push(delta);
                            } else {
                                let mut delta = Delta {
                                    operation: Operation::Delete,
                                    path: path_str.clone(),
                                    old_value: left.clone(),
                                    new_value: Value::Null,
                                    hash: String::default(),
                                };
                                delta.hash = calculate_hash(&delta).to_string();
                                deltas.push(delta);
                            }
                        }
                    }

                    seen.insert(path_str.clone(), true);
                }
            }
        }
    }
}
