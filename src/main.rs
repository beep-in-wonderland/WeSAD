// standard library imports
use std::fs::{exists, read_dir, File};
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::env::args;
use std::str::FromStr;
use std::sync::{Arc,Mutex,MutexGuard};
use std::net::TcpListener;
use std::thread;

// all crates for the project
use daemonize;
use mail_send; // SMTP email sending
use sqlite; // managing persistent data
use json; // parsing config files
use thiserror; // error macros
use fern; // logging
use chrono; // timing
use fancy_regex;

use log::{info,warn,debug,error,Level};


// separate custom modules
mod database; // Database management module using SQLite3
mod daemon;
mod discord;
mod smtp;
mod errors;
mod webhook;

// Relevant paths for folders 
pub struct ProgramPaths {
    log: PathBuf,
    persist: PathBuf,
    etc: PathBuf,
}


impl ProgramPaths {

    // Initializes the paths for the program files based on whether this executable
    // is running from a dev build folder or on a real machine.
    fn init(is_dev_build: bool) -> Self {
        let log: PathBuf;
        let persist: PathBuf;
        let etc: PathBuf;

        match is_dev_build {
            true => {
                let root_dir = current_dir().unwrap();

                log = root_dir.join("dev/log/");
                persist = root_dir.join("dev/persist/");
                etc = root_dir.join("dev/etc/")
            }

            false => {
                log = PathBuf::from("/var/log/bbnd/");
                persist = PathBuf::from("/var/lib/bbnd/");
                etc = PathBuf::from("/etc/")
            }
        }

        debug!("Log path: {0}", log.display());
        debug!("Persist path: {0}", persist.display());
        debug!("Etc path: {0}", etc.display());
        

        Self { 
            log, 
            persist, 
            etc, 
        }
    }
}

// Public struct to hold all args passed during CLI initialization of the service.
pub struct CLIArgs {
    log_level : Level,
}

impl Default for CLIArgs {
    fn default() -> Self {
        Self {
            log_level : Level::Debug,
        }
    }
}


fn main() -> Result<(), errors::Error> {

    // Allow the CLI args to be mutable in case these require adjustment during runtime 
    // via user input.
    let mut cli_args = parse_cli_args().unwrap(); 

    // Important runtime variables
    let is_dev_build: bool = check_dev_build().unwrap();
    let paths: ProgramPaths = ProgramPaths::init(is_dev_build);

    // Initialize the logger
    init_logger(&cli_args.log_level, &paths.log);

    // Initialize the daemon
    let mut daemon = daemon::Daemon::new(paths).expect("An unexpected error occurred initializing the daemon.");

    // Run the daemon
    daemon.run()?;

    Ok(())

}


/// Check whether the program is running as a precompiled binary or if it is running
/// in the dev build. This is done by checking whether the dir ``../../state`` exists,
/// AND whether the file ``../../target/debug/bbnd`` exists.
fn check_dev_build() -> Result<bool, errors::Error> {

    debug!("Checking for dev build...");
    // While in the dev build, the relative path of the binary will be ~/target/debug/
    // must be mut because the .pop() requires a mutable reference to the PathBuf
    let root_dir: PathBuf = current_dir().unwrap();

    // root_dir.join() will return a new owned PathBuf with the desired path appended, this way
    // the original root_dir remains unmutated.
    // Check for the dev build folders emulating the linux filesystem
    if root_dir.join("dev/etc").is_dir() 
        && root_dir.join("dev/log").is_dir() 
        && root_dir.join("dev/persist").is_dir() 
        && exists(root_dir.join("target/debug/bbnd")).unwrap()
    {
        // If this is running from the debug directory, the relative folder "~/state/" will exist
        // and is functionally the same as /usr/lib/bbnd/
        debug!("Dev build: TRUE");
        return Ok(true);
    }

    debug!("Dev build: FALSE");
    return Ok(false)
}


fn parse_cli_args() -> Result<CLIArgs, errors::Error> {
    
    let mut cli_args = CLIArgs::default();

    let args: Vec<String> = args().collect();

    // Main argument loop - iterating in keyword pairs
    for i in (1 .. args.len()).step_by(2) {
        dbg!(&args[i], &args[i+1]);
        if args[i] == "-l"
            || args[i] == "--log-level"{
            cli_args.log_level = log::Level::from_str(&args[i+1])
                .expect("Error - Invalid Log Level. Please select from ERROR, WARN, INFO, DEBUG");
        }
    }

    Ok(cli_args)
}


fn init_logger(log_level: &Level, log_path: &PathBuf) {

    let log_path: &PathBuf = &log_path.join("bbnd.log");
    if !exists(log_path).unwrap() {
        let _ = File::create_new(log_path);
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level.to_level_filter())
        .chain(std::io::stdout())
        .chain(fern::log_file(log_path).unwrap())
        .apply()
        .unwrap();

    debug!("Logger initialized")
}

