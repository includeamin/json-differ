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

impl Hash for Delta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.operation.hash(state);
        self.path.hash(state);
        self.old_value.to_string().hash(state);
        self.new_value.to_string().hash(state);
    }
}
