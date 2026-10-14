// standard library imports
use std::fs::{exists, File};
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::env::args;
use std::range;
use std::str::FromStr;

// all crates for the project
use tokio; // task scheduling
use daemonize;
use dbus; // communication with external programs
use mail_send; // SMTP email sending
use sqlite; // managing persistent data
use json; // parsing config files
use thiserror;
use fern;
use chrono;


use log::{LevelFilter,info,warn,debug,error,Level};


// separate custom modules
mod bbdb; // Database management module using SQLite3
mod bbnd;
mod bbconf;
mod discord;
mod smtp;
mod errors;


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


        Self { 
            log, 
            persist, 
            etc, 
        }
    }
}

// Public struct to hold all args passed during CLI initialization of the service.
pub struct CLIArgs {
    log_level: Level,
}


fn main() {

    let mut cli_args = parse_cli_args().unwrap();
    let is_dev_build: bool = check_dev_build().unwrap();
    let paths: ProgramPaths = ProgramPaths::init(is_dev_build);
    init_logger(&cli_args.log_level, paths.log);



    // read, validate, and store configs (bbconf.rs)
    // init & check connection to persist db (bbdb.rs)
    //      validate data within persist db (bbdb.rs)
    // test connections to 
    //      SMTP (smtp.rs)
    //      Discord webhooks (discord.rs) 

    // initialize daemon with validated data
    //      daemon publishes to session D-Bus
    
    // external scripts use dbus method calls to send updates
    //      bbnd updates the stored data in the database
    // on set cron intervals as defined in bbnd.d/myfile.conf, the daemon
    //   will publish updates to the webhooks and/or send emails 


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
    let mut log_level: Level = Level::Warn;  

    let args: Vec<String> = args().collect();

    // Main argument loop - iterating in keyword pairs
    for i in (1 .. args.len()).step_by(2) {
        if args[i] == "-l"
            || args[i] == "--log-level"{
            log_level = log::Level::from_str(&args[i+1]).unwrap();
        }
    }

    Ok(CLIArgs {
        log_level,
    })
}


fn init_logger(log_level: &Level, log_path: PathBuf) {

    let log_path: &PathBuf = &log_path.join("bbnd.log");
    if !exists(log_path).unwrap() {
        dbg!("File not found");
        dbg!(log_path);
        let _ = File::create_new(log_path);
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}\n",
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

    log::debug!("Logger initialized")
}


