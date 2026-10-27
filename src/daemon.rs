use super::*;

use std::io::Read;
use std::ops::Not;
use std::str::FromStr;
use std::thread;

use crate::database::Session;
use crate::errors::Error::*;
use crate::read_dir;
use crate::PathBuf;
use crate::File;

use crate::fancy_regex::Regex;
use crate::webhook;
use crate::{Arc,Mutex};
use crate::logging::{info,error,debug,warn};
use crate::{database, errors};
use crate::json;

use chrono;
use fancy_regex::Captures;
use json::JsonValue;


pub mod io;
pub(super) mod listener;
pub(super) mod updater;

#[cfg(test)]
mod tests;

const PATH_WEBHOOKS_ETC: &str = "bbnd/webhooks.d";


/// Regex capture which will split a fully qualified file path with extension into three non-overlapping captures:
/// example: ``/this/is/my/file.conf``
/// `dir = /this/is/my/`
/// `name = file`
/// `ext = .conf`
const REGEX_CONF: &str = "(?<dir>^[(?:\\/|\\\\)0-9a-zA-Z_\\- ]+(?:\\.d)?(?:\\/|\\\\))(?<name>[0-9a-zA-Z_\\-. ]+)(?<ext>\\.conf)$";
const REGEX_PLACEHOLDERS: &str = "(\\$\\$(?<flag>[\\w]+)\\$\\$)";


struct WebhookConfigContents {
    name : String,
    url : String,
    webhook_type: webhook::Type,
    interval: String,
    period: i64,
}


struct RuntimeConfig {
    listener_addr : String,
    listener_port : String,
    smtp_user: String,
    smtp_target: String,
    smtp_host: String,
    smtp_port: String,
}


impl RuntimeConfig {
    fn new(paths: &io::ProgramPaths) -> Self {

        let json_str = io::open_as_string(
            &paths.get_etc().to_str().unwrap().to_string(),
            &"bbnd".to_string(),
             &".conf".to_string()
        ).unwrap_or(
            // If the file wasn't found, generate an empty json str so defaults get populated
            "{\"never gonna\" : \"give you up\"}".to_string() 
        );

        let config = json::parse(json_str.as_str()).expect("Either config was found or blank was generated");

        let result = RuntimeConfig { 
            listener_addr: RuntimeConfig::has_key_or_default(
                &config, vec!["listener_addr"]), 
            listener_port: RuntimeConfig::has_key_or_default(
                &config, vec!["listener_port"]), 
            smtp_user: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_user"]), 
            smtp_target: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_target"]), 
            smtp_host: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_host"]), 
            smtp_port: RuntimeConfig::has_key_or_default(
                &config, vec!["smtp_config", "smtp_port"]), 
        };

        debug!("{}", result);
        result
    }


    fn has_key_or_default(jsonval: &JsonValue, keys: Vec<&str>) -> String {

        let mut target: JsonValue = jsonval.clone();

        for key in keys {
            if target.has_key(key) {
                target = target[key].clone();
            } else {
                return match key {
                    "listener_addr" => "127.0.0.1",
                    "listener_port" => "22000",
                    _               => ""
                }.to_string()
            }
        }
        return target.to_string()

    }
}

impl Display for RuntimeConfig {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RuntimeConfig:
        listener-ip: {}:{}
        smtp user: {}
        smtp target: {}
        smtp host: {}:{}", 
        self.listener_addr, self.listener_port, 
        self.smtp_user, self.smtp_target,
        self.smtp_host, self.smtp_port)
    }
}



pub struct Daemon {
    paths : io::ProgramPaths,
    session : Arc<Mutex<database::Session>>,
    runtime_config: RuntimeConfig,
}


impl Daemon {

    /// Initialization function of the daemon only requires that the paths be specified for this runtime.
    pub fn new(is_dev_build: bool, log_level: &logging::Level) -> Result<Self, errors::Error> {
        // TODO Rewrite this as a builder eventually with .with_dev_build() as a buildparam
        let paths = io::ProgramPaths::get_paths(is_dev_build);

        logging::init_logger(log_level, paths.get_log());

        // Initialize persistent local db.
        let session = 
            database::Session::init(&paths.get_persist())
            .expect("An unexpected error occurred initializing the database.");
        
        let config = RuntimeConfig::new(&paths);

        


        Ok (Self { 
                paths,
                session : Arc::new(Mutex::new(session)),
                runtime_config : config,
            })
    }

    pub fn run(&mut self) -> Result<(), errors::Error> {

        // Read configs and store data in db
        self.parse_all_webhook_configs()?;

        // Lock the session mutex and drop all orphaned values

        let session = self.session.lock().expect("poisoned");
        database::write::drop_orphaned_values(&session)?;
        drop(session);


        // Spawns a thread to continuously update webhooks with current database values
        let updater_handle = updater::run(Arc::clone(&self.session)).expect("Program does nothing if this fails");

        // Spawns a thread to continuously listen for & execute database updates
        let listener_handle = listener::run(
            Arc::clone(&self.session), &self.runtime_config)
            .expect("Program does nothing if this fails");

        // TODO On a failed thread, catch the unwind, report the error to the logger, and then try and restart the thread?
        updater_handle.join().unwrap();
        listener_handle.join().unwrap();

        Ok (())
    }


    fn parse_all_webhook_configs(&mut self) -> Result<(), errors::Error> {
        log::debug!("Starting webhook parse");
        let webhooks = read_dir(&self.paths.get_etc().join(PATH_WEBHOOKS_ETC))?;
        let mut webhook_path: PathBuf;

        let regex_conf = Regex::new(REGEX_CONF)?;
        let mut captures: Option<Captures<'_, String>> ;
        let mut path: String;
        let mut name: String;

        let mut webhook_config: WebhookConfigContents;
        let mut webhook_placeholders: Vec<String>;


        for webhook in webhooks {
            webhook_path = webhook.unwrap().path();

            let target_string = webhook_path.display().to_string();
            captures = regex_conf.captures(&target_string)?;


            // If the regex found a capture, it means it must have been a .conf file
            if captures.is_none().not() {
                log::debug!("Config file found: {}", target_string);

                // Extract the 'path' & 'filename' groups from the regex capture
                path = captures.as_ref().unwrap().get(1).unwrap().as_str().to_string();
                name = captures.as_ref().unwrap().get(2).unwrap().as_str().to_string();

                webhook_config = io::parse_webhook_config(&path, &name)?;
                webhook_placeholders = io::parse_webhook_json_placeholders(&path, &name)?;

                // Calculate the period, in seconds, based on `interval` and `period`
                let period= webhook::Period::from_type_value(
                   &webhook_config.interval, webhook_config.period);

                log::debug!("Creating webhook descriptor for {}", name);
                let descriptor: webhook::Descriptor = webhook::Descriptor {
                    name : webhook_config.name.clone(),
                    url : webhook_config.url,
                    webhook_type : webhook_config.webhook_type,
                    path,
                    period : period
                };
                
                let session = self.session.lock().expect("poisoned");

                database::write::init_webhook_parent(&session, &descriptor)?;
                database::write::init_webhook_placeholders(&session, &webhook_config.name, &webhook_placeholders)?;

            }

        }

        debug!("Finished webhook parse");
        Ok(())
    }


}


fn validate_config_lines(json_data: &JsonValue, name: &String) -> Result<(), errors::Error> {

    if json_data["url"].is_string() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook url.", &name));
    }

    if json_data["type"].is_string() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook type. Choose \"status\" or \"alert\".", &name));
    }

    if json_data["interval"].is_string() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook interval type. Choose \"seconds\", \"minutes\", or \"hours\".", &name));
    }

    if json_data["period"].is_number() { 
    } else {
        ConfigErr(format!("{}.conf: invalid webhook interval value. Choose an integer.", &name));
    }

    Ok(())
}