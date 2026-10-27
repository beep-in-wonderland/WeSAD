// standard library imports
use std::fs::{exists, read_dir, File};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::str::FromStr;
use std::sync::{Arc,Mutex};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};
use std::collections::HashMap;

// all crates for the project
use daemonize;
use sqlite; // managing persistent data
use json; // parsing config files
use thiserror; // error macros
use fern; // logging
use chrono; // timing
use fancy_regex;
use reqwest;
use axum;
use serde;


pub mod daemon;
pub mod client;
pub mod errors;
pub mod logging;
mod database;
mod webhook;