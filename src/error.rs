#[derive(Debug)]
pub struct Error {
    pub error_message: String,
}

impl Error {
    pub fn new(error_message: &str) -> Self {
        log::error!("{}", error_message.to_lowercase());

        Self {
            error_message: error_message.to_string(),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.error_message
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error {
            error_message: error.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Error {
        Error {
            error_message: error.to_string(),
        }
    }
}
