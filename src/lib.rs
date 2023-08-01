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

fn diff(
    a: &Value,
    b: &Value,
    current_path: &mut Vec<String>,
    paths: &mut Vec<String>,
    deltas: &mut Vec<Delta>,
    seen: &mut HashMap<String, bool>,
    reverse: bool,
) {
    match a {
        Value::Object(map) => {
            for (key, value) in map.iter() {
                if current_path.is_empty() {
                    current_path.push(format!("$.{}", key));
                } else {
                    current_path.push(key.to_string());
                }
                diff(value, b, current_path, paths, deltas, seen, reverse);
                current_path.pop();
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                match current_path.len() {
                    0 => current_path.push(format!("$[{}]", index)),
                    _ => current_path.push(format!("[{}]", index)),
                }
                diff(value, b, current_path, paths, deltas, seen, reverse);
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
            let b_value = parsed_path.query(b).first();
            match b_value {
                // the value exists in both, so we need to check if it's the same
                Some(b_value) => {
                    if a != b_value {
                        deltas.push(Delta {
                            operation: Operation::Change,
                            path: path.clone(),
                            old_value: a.clone(),
                            new_value: b_value.clone(),
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
                            new_value: a.clone(),
                        });
                    } else {
                        deltas.push(Delta {
                            operation: Operation::Delete,
                            path: path.clone(),
                            old_value: a.clone(),
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
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn diff_success() {
        // given
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/testdata/small_json_1.json");
        let data = fs::read_to_string(d).expect("Unable to read file");
        let json_a = serde_json::from_str(&data).expect("Unable to parse");

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/testdata/small_json_2.json");
        let data = fs::read_to_string(d).expect("Unable to read file");
        let json_b = serde_json::from_str(&data).expect("Unable to parse");

        let mut a_paths: Vec<String> = Vec::new();
        let mut deltas: Vec<Delta> = Vec::new();
        let mut a_current_path: Vec<String> = Vec::new();
        let mut seen: HashMap<String, bool> = HashMap::new();

        // when
        diff(
            &json_a,
            &json_b,
            &mut a_current_path,
            &mut a_paths,
            &mut deltas,
            &mut seen,
            false,
        );

        diff(
            &json_b,
            &json_a,
            &mut a_current_path,
            &mut a_paths,
            &mut deltas,
            &mut seen,
            true,
        );

        // then
        assert_eq!(deltas.len(), 4, "Expected 4 deltas, got {}", deltas.len());

        assert_eq!(deltas[0].operation, Operation::Change);
    }
}
