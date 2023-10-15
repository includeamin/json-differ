use crate::patcher::PatchOptions;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn is_multi_dimensional_array(json_path: &str) -> (bool, usize, Vec<usize>) {
    if !json_path.contains('[') {
        return (false, 0, Vec::new());
    }

    let array_index_regex = Regex::new(r"\[(\d+)]").unwrap();

    let indices: Vec<usize> = array_index_regex
        .captures_iter(json_path)
        .filter_map(|capture| capture.get(1).and_then(|m| m.as_str().parse().ok()))
        .collect();

    let indices_len = indices.len();
    let is_multi_dimensional = indices_len > 0;

    (is_multi_dimensional, indices_len - 1, indices)
}

#[derive(Debug)]
pub struct PathItem {
    pub key: String,
    pub is_array: bool,
    pub indices: Vec<usize>,
    pub is_last: bool,
}

impl PathItem {
    pub fn new(path: String, is_array: bool, indices: Vec<usize>, is_last: bool) -> Self {
        PathItem {
            key: get_key(path.as_str()),
            is_array,
            indices,
            is_last,
        }
    }
}

pub fn analyse_path(json_path: &str) -> Vec<PathItem> {
    let split_path = json_path.split('.');
    let mut path_items: Vec<PathItem> = vec![];
    let total_path_count = split_path.clone().count();

    for (index, path) in split_path.into_iter().enumerate() {
        let (is_array, _, indices) = is_multi_dimensional_array(path);
        let is_last = index == total_path_count - 1;

        let path_item = PathItem::new(path.to_string(), is_array, indices, is_last);

        path_items.push(path_item);
    }

    path_items
}

#[warn(dead_code)]
pub fn create_multi_dimensional_vec(dim_count: usize) -> Value {
    let mut vec = Vec::new();
    let mut current_vec = &mut vec;
    for _ in 0..dim_count {
        current_vec.push(Value::Array(Vec::new()));
        current_vec = current_vec.last_mut().unwrap().as_array_mut().unwrap();
    }

    Value::Array(vec)
}

pub fn access_element<'a>(array: &'a Vec<Value>, indices: &[usize]) -> Option<&'a Value> {
    if indices.is_empty() {
        return None;
    }

    let index = indices.first().unwrap();

    if *index >= array.len() {
        return None;
    }

    let element = &array[*index];

    if let Value::Array(arr) = element {
        return access_element(arr, &indices[1..]);
    }

    Some(element)
}

pub fn insert_element(array: &mut Vec<Value>, indices: &[usize], value: Value) {
    if indices.is_empty() {
        return;
    }

    let index = indices.first().unwrap();

    if indices.len() == 1 {
        array.insert(*index, value);
        return;
    }

    if *index >= array.len() {
        array.push(json!([]));
    }

    let element = &mut array[*index];

    if let Value::Array(arr) = element {
        return insert_element(arr, &indices[1..], value);
    }

    *element = value;
}

pub fn change_element(array: &mut Vec<Value>, indices: &[usize], value: Value) {
    if indices.is_empty() {
        return;
    }

    let index = indices.first().unwrap();

    if *index >= array.len() {
        if indices.len() == 1 {
            array.insert(*index, value);
            return;
        }
        return;
    }

    let element = &mut array[*index];

    if let Value::Array(arr) = element {
        return change_element(arr, &indices[1..], value);
    }

    *element = value;
}

pub fn remove_element(array: &mut Vec<Value>, indices: &[usize], options: PatchOptions) {
    if indices.is_empty() {
        return;
    }

    let index = indices.first().unwrap();

    if *index >= array.len() {
        return;
    }

    let element = &mut array[*index];

    if let Value::Array(arr) = element {
        // If we are removing the last element of an array, and the array is the only element of the parent array,
        if indices.len() == 2 && arr.len() == 1 && options.omit_empty {
            array.remove(*index);
            return;
        }

        return remove_element(arr, &indices[1..], options);
    }

    array.remove(*index);
}

fn get_key(key: &str) -> String {
    if !key.contains('[') {
        return key.to_string();
    }

    let start = key.find('[');
    key[..start.unwrap()].to_string()
}
