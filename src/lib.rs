// standard library imports
use std::fs::{exists, read_dir, File};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::str::FromStr;
use std::sync::{Arc,Mutex};
use std::thread::{self, JoinHandle};
use std::collections::HashMap;
use std::ops::Not;
use std::io::Read;

// all crates for the project
use sqlite; // managing persistent data
use json; // parsing config files
use thiserror; // error macros
use fern; // logging
use chrono; // timing
use fancy_regex;
use reqwest;
use axum;
use serde;
use directories;


pub mod daemon;
pub mod io;
pub mod client;
pub mod errors;
pub mod logging;
mod database;
mod webhook;