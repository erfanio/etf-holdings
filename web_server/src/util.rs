use std::fmt::{Debug, Display};

// Generic error for other stuff. Implementing From on Error would cause everything to have generic messages and kind
// so this is here to clearly distinguish between generic errors and errors with good messages
#[derive(Debug)]
pub struct AnyError(String);

impl<T: Display> From<T> for AnyError {
    fn from(error: T) -> Self {
        AnyError(error.to_string())
    }
}

// Error type that emits good HTTP status
#[derive(Debug)]
pub enum Error {
    Generic(String),
    NotFound(String),
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        warn_!("Error: {:?}", self);
        match self {
            Error::Generic(_) => Err(rocket::http::Status::InternalServerError),
            Error::NotFound(_) => Err(rocket::http::Status::NotFound),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
