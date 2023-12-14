use crate::utils::calculate_hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Operation {
    Add,
    Change,
    Delete,
}

impl Hash for Operation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Operation::Add => {
                "Add".hash(state);
            }
            Operation::Change => "Change".hash(state),
            Operation::Delete => {
                "Delete".hash(state);
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub operation: Operation,
    pub path: String,
    pub old_value: Value,
    pub new_value: Value,
    pub hash: String,
}

impl Delta {
    pub fn new(operation: Operation, path: String, old_value: Value, new_value: Value) -> Self {
        let mut delta = Delta {
            operation,
            path,
            old_value,
            new_value,
            hash: String::default(),
        };
        delta.hash = calculate_hash(&delta).to_string();
        delta
    }
}

impl Hash for Delta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.operation.hash(state);
        self.path.hash(state);
        self.old_value.to_string().hash(state);
        self.new_value.to_string().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_hash() {
        let delta = Delta::new(
            Operation::Add,
            "$.test".to_string(),
            Value::Null,
            Value::Number(1.into()),
        );
        assert_eq!(delta.hash, "13989947290824433245");
    }
}
