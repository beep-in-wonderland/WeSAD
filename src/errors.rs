use crate::thiserror;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sqlite failed: {0}")]
    SqliteErr(#[from] sqlite::Error),

    #[error("std io error: {0}")]
    StdIoErr(#[from] std::io::Error),

    #[error("regex error: {0}")]
    RegexErr(#[from] fancy_regex::Error),

    #[error("Json error: {0}")]
    JsonErr(#[from] json::Error),
    
    #[error("Database unreadable")]
    BbdbUnreadable(),

    #[error("Invalid argument: {0}")]
    ArgumentErr(String),

    #[error("Config Error: {0}")]
    ConfigErr(String),

    #[error("Reqwest Error: {0}")]
    ReqwErr(#[from] reqwest::Error),

    #[error("HTTP Error: {0}")]
    HttpErr(String),
}