use std::error;
// standard library imports
use std::fs::exists;
use std::path::PathBuf;
use std::env::{current_dir, args};
use std::str::FromStr;

use daemonize::Daemonize;
use directories;


// lib import
use bbnd::{
    self,
    logging::Level,
};


// Public struct to hold all args passed during CLI initialization of the service.
pub struct CLIArgs {
    log_level : Level,
    daemonize: bool,
}

impl Default for CLIArgs {
    fn default() -> Self {
        Self {
            log_level : Level::Info,
            daemonize : false,
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

    if cli_args.daemonize {
        println!("Daemonizing process...");
        match daemonize(){
            Ok(_) => (),
            Err(e) => panic!("Failed to daemonize: {e}"),
        };
    }
    
    // Initialize the service
    let mut service = bbnd::service::Service::new(
        is_dev_build,
        &cli_args.log_level,
    )
    .expect("An unexpected error occurred initializing the daemon.");

    service.run().expect("Somethin real bad happened");

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
        && root_dir.join("dev/lib").is_dir() 
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
    for i in 1 .. args.len() {
        match args[i].as_str() {
            "-l" | "--log-level" => {
                if i < args.len() {
                    cli_args.log_level = log::Level::from_str(&args[i+1])
                        .expect("Error - Invalid Log Level. Please select from ERROR, WARN, INFO, DEBUG");
                }
            },
            "-d" | "--detach" => {
                cli_args.daemonize = true
            }
            _ => ()
        };
    }

    Ok(cli_args)
}


fn daemonize() -> Result<String, bbnd::errors::Error> {

    let dirs = directories::ProjectDirs::from(
            "com", "beepdodev", "bbnd")
            .expect("Failed to find ENV directories");

    let dir = dirs.runtime_dir().unwrap();
    std::fs::create_dir_all(dir)?;

    let daemonize = Daemonize::new()
        .pid_file(&dir.join("daemon.pid"))
        .chown_pid_file(false) // is optional, see `Daemonize` documentation
        .working_directory(&dir) // for default behaviour.
        .umask(0o022)    // Set umask, `0o027` by default.
        .privileged_action(|| "Executed before drop privileges");

        match daemonize.start() {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(bbnd::errors::Error::DaemErr(e))
        }
    }