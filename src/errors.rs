use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ProcessError {
    #[display(fmt = "Validation error on field: {} with message: {}", field, message)]
    ValidationError { field: String, message: String },

    #[display(fmt = "Unknown error: {}", message)]
    UnknownError { message: String },

    #[display(fmt = "Conflict: {}", message)]
    Conflict { message: String },
}
