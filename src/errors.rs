use derive_more::{Display, Error};

#[derive(Display, Error, Debug)]
pub enum ProcessError {
    #[display(fmt = "Unknown error: {}", message)]
    Unknown { message: String },
}
