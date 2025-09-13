use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    Marshal { cause: String },
    HeaderLen { cause: String },
    Pipe { cause: String },
    PrefixValidation { cause: String },
    CRCVerification { cause: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CRCVerification { cause } => {
                write!(
                    f,
                    "validation failed on the message sent to STDOUT, cause {}",
                    cause
                )
            }
            Error::Marshal { cause } => write!(f, "payload marshaling failed: {}", cause),
            Error::HeaderLen { cause } => write!(f, "incorrect len, cause: {}", cause),
            Error::Pipe { cause } => write!(f, "pipe send error, cause: {}", cause),
            Error::PrefixValidation { cause } => {
                write!(f, "prefix validation error: {}", cause)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Pipe {
            cause: error.to_string(),
        }
    }
}
