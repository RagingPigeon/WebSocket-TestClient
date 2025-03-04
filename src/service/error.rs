#[derive(Debug)]
pub struct CommonError {
    pub message: String,
}

impl std::fmt::Display for CommonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommonError {}

impl CommonError {
    pub fn from(new_message: &str) -> CommonError {
        CommonError {
            message: String::from(new_message),
        }
    }

    pub fn from_string(new_message: String) -> CommonError {
        CommonError {
            message: new_message,
        }
    }
}