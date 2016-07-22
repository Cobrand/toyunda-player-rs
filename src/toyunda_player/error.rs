use std::error::Error as StdError;
use mpv::Error as MpvError;
pub type Result<T> = ::std::result::Result<T,Error>;

#[derive(Clone,Debug)]
pub enum Error {
    Text(String),
    FileNotFound(String),
    MpvError(MpvError),
    UnknownError
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Text(ref string) => string.as_str(),
            Error::MpvError(ref mpv_error) => mpv_error.description(),
            Error::FileNotFound(ref file) => "given file was not found",
            Error::UnknownError => "unknown error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Text(ref string) => None,
            Error::MpvError(ref mpv_error) => Some(mpv_error),
            Error::FileNotFound(ref file) => None,
            Error::UnknownError => None,
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.description())
    }
}