use std::error;
// standard library imports
use std::fs::{exists, read_dir, File};
use std::path::{Path, PathBuf};
use std::env::current_dir;
use std::env::args;
use std::str::FromStr;
use std::net::TcpListener;
use std::thread::{self, JoinHandle};



// lib import
use bbnd::{
    self,
    logging::Level,
};





// Public struct to hold all args passed during CLI initialization of the service.
pub struct CLIArgs {
    pub log_level : Level,
}

impl Default for CLIArgs {
    fn default() -> Self {
        Self {
            log_level : Level::Info,
        }
    }
}


fn main() -> () {

    // Allow the CLI args to be mutable in case these require adjustment during runtime 
    // via user input.
    let cli_args = parse_cli_args().unwrap(); 

    // Check if the current binary is running in a dev environment
    let is_dev_build: bool = check_dev_build()
        .unwrap_or(true);
    
    // Initialize the daemon (it's not a daemon yet lol)
    let mut daemon = bbnd::daemon::Daemon::new(
        is_dev_build,
        &cli_args.log_level,
    )
    .expect("An unexpected error occurred initializing the daemon.");

    // Run the (not) daemon. This call will join on one of the spawned threads that loops indefinitely.
    // TODO add some logic to restart the daemon on a thread failure, as well as have the threads 
    // check with each other when one fails.
    daemon.run().expect("Somethin real bad happened");

}


/// Check whether the program is running as a precompiled binary or if it is running
/// in the dev build. This is done by checking whether some of the various dev directories
/// exist.
fn check_dev_build() -> Result<bool, Box<dyn error::Error>> {

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
        return Ok(true);
    }

    return Ok(false)
}


fn parse_cli_args() -> Result<CLIArgs, Box<dyn error::Error>> {
    
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


