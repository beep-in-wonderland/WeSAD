use crate::thiserror;

/// Dear god forgive me for my sins. I will atone for this (fix it) some day (TODO)
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

    #[error("Reqwest Error: {0}")]
    ReqwErr(#[from] reqwest::Error),

    #[error("HTTP Error: {0}")]
    HttpErr(String),

    #[error("Daemonize Error: {0}")]
    DaemErr(#[from] daemonize::Error),

    #[error("Config Error: {0}")]
    ConfigErr(String),

    #[error("Client Error: {0}")]
    ClientErr(String),
}