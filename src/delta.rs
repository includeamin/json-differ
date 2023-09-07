use serde_json::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add,
    Change,
    Delete,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Delta {
    pub operation: Operation,
    pub path: String,
    pub old_value: Value,
    pub new_value: Value,
}
