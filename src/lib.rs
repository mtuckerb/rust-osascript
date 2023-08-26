use std::process;
use std::io;
use std::fmt;
use std::string::FromUtf8Error;
use std::io::Write;
use std::error;

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// The error from the script system
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Script(String),
}

/// Holds an apple flavoured JavaScript
pub struct JavaScript {
    code: String,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Script(format!("UTF-8 Error: {}", err))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Json(ref err) => err.description(),
            Error::Script(..) => "script error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "script io error: {}", err),
            Error::Json(ref err) => write!(f, "script json error: {}", err),
            Error::Script(ref msg) => write!(f, "script error: {}", msg),
        }
    }
}

#[derive(Serialize)]
struct EmptyParams {}

fn wrap_code<S: Serialize>(code: &str, params: S) -> Result<String, Error> {
    let mut buf: Vec<u8> = vec![];
    write!(&mut buf, "var $params = ")?;
    serde_json::to_writer(&mut buf, &params)?;
    write!(&mut buf, ";JSON.stringify((function() {{{};return null;}})());", code)?;
    Ok(String::from_utf8(buf)?)
}

//----------------------
/// Holds an apple flavoured JavaScript
impl JavaScript {
    /// Creates a new script from the given code.
    pub fn new(code: &str) -> JavaScript {
        JavaScript {
            code: code.to_string(),
        }
    }

    /// Executes the script and does not pass any arguments.
    pub fn execute<'a, D: DeserializeOwned>(&self) -> Result<D, Error> {
        self.execute_with_params(EmptyParams {})
    }

    /// Executes the script and passes the provided arguments.
    pub fn execute_with_params<'a, S: Serialize, D: DeserializeOwned>(&self, params: S)
    -> Result<D, Error>
    {
        let wrapped_code = wrap_code(&self.code, params)?;
        let output = process::Command::new("osascript")
            .arg("-l")
            .arg("JavaScript")
            .arg("-e")
            .arg(&wrapped_code)
            .output()?;
        if output.status.success() {
            Ok(serde_json::from_slice(&output.stdout)?)
        } else {
            Err(Error::Script(String::from_utf8(output.stderr)?))
        }
    }
}

pub struct AppleScript {
    code: String,
}

impl AppleScript {
    /// Creates a new script from the given code.
    pub fn new(code: &str) -> AppleScript{
        AppleScript {
            code: code.to_string(),
        }
    }

    /// Executes the script and passes the provided arguments.
    pub fn execute<'a, D: DeserializeOwned>(&self )
    -> Result<D, Error>
    {
        let wrapped_code = &self.code;
        let output = process::Command::new("osascript")
            .arg("-l")
            .arg("AppleScript")
            .arg("-e")
            .arg(&wrapped_code)
            .output()?;
        if output.status.success() {
            Ok(serde_json::from_slice(&output.stdout)?)
        } else {
            Err(Error::Script(String::from_utf8(output.stderr)?))
        }
    }
}
