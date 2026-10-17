use crate::thiserror;
use crate::sqlite::Error as sqliteError;


#[derive(Debug)]
pub struct ErrorObj {
    code: Option<isize>,
    message: Option<String>,
}

impl ErrorObj {
    fn as_display(&self) -> String {
        return format!("Error code-{0:?}: {1:?}", self.code, self.message)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sqlite failed: {0}")]
    SqliteErr(#[from] sqliteError),

    #[error("std io error: {0}")]
    StdIoErr(#[from] std::io::Error),

    #[error("regex error: {0}")]
    RegexErr(#[from] regex::Error),

    #[error("Json error: {0}")]
    JsonErr(#[from] json::Error),
    
    #[error("bbdb unreadable")]
    BbdbUnreadable(ErrorObj),

    #[error("Invalid argument: {0}")]
    ArgumentError(ErrorObj),
}