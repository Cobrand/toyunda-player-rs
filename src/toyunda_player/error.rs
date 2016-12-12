use std::error::Error as StdError;
use mpv::Error as MpvError;
use serde_json::error::Error as SerdeJsonError;
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    Text(String),
    FileNotFound(String),
    MpvError(MpvError),
    JsonError(SerdeJsonError),
    UnknownError,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Text(ref string) => string.as_str(),
            Error::MpvError(ref mpv_error) => mpv_error.description(),
            Error::FileNotFound(_) => "given file was not found",
            Error::JsonError(ref e) => e.description(),
            Error::UnknownError => "unknown error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Text(_) => None,
            Error::MpvError(ref mpv_error) => Some(mpv_error),
            Error::JsonError(ref serde_json_error) => Some(serde_json_error),
            Error::FileNotFound(_) => None,
            Error::UnknownError => None,
        }
    }
}

impl From<MpvError> for Error {
    fn from(e: MpvError) -> Error {
        Error::MpvError(e)
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::Text(ref string) => write!(f, "{}", string),
            Error::MpvError(ref mpv_error) => write!(f, "Error from Mpv : {}", mpv_error),
            Error::FileNotFound(ref e) => write!(f, "File {} not found", e),
            Error::JsonError(ref e) => write!(f, "JSON Error : {}", e),
            Error::UnknownError => write!(f, "Unknown Error"), 
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Text(s)
    }
}

impl From<SerdeJsonError> for Error {
    fn from(s: SerdeJsonError) -> Error {
        Error::JsonError(s)
    }
}
