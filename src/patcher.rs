use crate::delta::{Delta, Operation};
use crate::errors::ProcessError;
use crate::utils::{
    analyse_path, change_element, insert_element, remove_element, remove_empty_levels,
};
use serde_json::{json, Value};

#[derive(Clone, Copy)]
pub struct PatchOptions {
    pub force: bool,
    pub omit_empty: bool,
}

impl Default for PatchOptions {
    fn default() -> Self {
        PatchOptions::new()
    }
}

impl PatchOptions {
    pub fn new() -> Self {
        PatchOptions {
            force: false,
            omit_empty: false,
        }
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn omit_empty(mut self, omit_empty: bool) -> Self {
        self.omit_empty = omit_empty;
        self
    }
}

pub fn patch(base: Value, deltas: &Vec<Delta>, options: PatchOptions) -> Value {
    let base_value = &mut base.clone();

    for delta in deltas {
        patch_by_path(
            base_value,
            delta.path.as_str(),
            &delta.new_value,
            delta.operation.clone(),
            options,
        )
        .unwrap()
    }

    if options.omit_empty {
        remove_empty_levels(base_value);
    }

    base_value.clone()
}

pub(crate) fn patch_by_path(
    json: &mut Value,
    path: &str,
    value: &Value,
    operation: Operation,
    options: PatchOptions,
) -> Result<(), ProcessError> {
    let mut current = json;
    let path_analysis = analyse_path(path);
    let mut paths = path_analysis.iter().enumerate();
    loop {
        let path_item = paths.next();

        if path_item.is_none() {
            return Ok(());
        }

        let path_item = path_item.unwrap();

        let (_index, item) = path_item;

        if item.key == "$" {
            continue;
        }

        if item.is_last && current.is_array() {
            if !item.is_array {
                let arr = current.as_array_mut().unwrap();
                let mut obj = json!({});
                obj.as_object_mut()
                    .unwrap()
                    .insert(item.key.as_str().to_string(), value.clone());
                arr.push(obj);
                return Ok(());
            }

            return Err(ProcessError::Unknown {
                message: "Unknown error".to_string(),
            });
        }

        // when we are in the last item, we need to do the actual operation
        // we have two cases here:
        // 1. the current item is an array
        // 2. the current item is an object
        if item.is_last && current.is_object() {
            // the last item is in an array
            // so we need to do the operation on the array
            if item.is_array {
                return match operation {
                    Operation::Add => {
                        // if the element was not in the object, we need to create it
                        let object = current.as_object_mut().unwrap();
                        if !object.contains_key(item.key.as_str()) {
                            object.insert(item.key.as_str().to_string(), Value::Array(vec![]));
                        }
                        current = &mut current[item.key.as_str()];
                        insert_element(
                            current.as_array_mut().unwrap(),
                            &item.indices,
                            value.clone(),
                        );
                        Ok(())
                    }
                    Operation::Change => {
                        current = &mut current[item.key.as_str()];
                        change_element(
                            current.as_array_mut().unwrap(),
                            &item.indices,
                            value.clone(),
                        );
                        Ok(())
                    }
                    Operation::Delete => {
                        current = &mut current[item.key.as_str()];
                        remove_element(current.as_array_mut().unwrap(), &item.indices, options);
                        Ok(())
                    }
                };
            }

            // the last item is an object
            // so we need to do the operation on the object
            return match operation {
                Operation::Add => {
                    // if the element was not in the object, we need to create it
                    let object = current.as_object_mut().unwrap();
                    if !object.contains_key(item.key.as_str()) {
                        object.insert(item.key.as_str().to_string(), Value::Null);
                    }

                    current[item.key.as_str()] = value.clone();
                    Ok(())
                }
                Operation::Change => {
                    current[item.key.as_str()] = value.clone();
                    Ok(())
                }
                Operation::Delete => {
                    current.as_object_mut().unwrap().remove(item.key.as_str());
                    Ok(())
                }
            };
        }

        // this logic is just for moving the current pointer to the right place
        match current {
            Value::Object(obj) => {
                if item.is_array {
                    // if the element was not in the object, we need to create it
                    if !obj.contains_key(item.key.as_str()) {
                        obj.insert(item.key.as_str().to_string(), json!([]));
                    }

                    current = &mut obj[item.key.as_str()];

                    for (_, _index) in item.indices.iter().enumerate() {
                        let array = current.as_array_mut().unwrap();

                        if (array.is_empty() || *_index > array.len())
                            && operation == Operation::Add
                        {
                            if item.indices.len() == 1 {
                                array.push(json!({}));
                                continue;
                            }

                            array.push(json!([]));
                            continue;
                        }
                        current = &mut current[*_index];
                    }

                    continue;
                }

                // if the element was not in the object, we need to create it
                if !obj.contains_key(item.key.as_str()) {
                    obj.insert(item.key.as_str().to_string(), json!({}));
                }

                current = &mut obj[item.key.as_str()];

                continue;
            }
            _ => {
                return Err(ProcessError::Unknown {
                    message: "Unknown error".to_string(),
                });
            }
        }
    }
}
