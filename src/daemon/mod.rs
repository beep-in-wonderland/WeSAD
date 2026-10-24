use core::fmt;
use std::io::Read;
use std::ops::Not;
use std::str::FromStr;
use std::thread;

use crate::database::Session;
use crate::errors::Error::ArgumentErr;
use crate::errors::Error::ConfigErr;
use crate::read_dir;
use crate::PathBuf;
use crate::File;

use crate::ProgramPaths;
use crate::fancy_regex::Regex;
use crate::webhook;
use crate::{Arc,Mutex};
use crate::{info,error,debug,warn};
use crate::{smtp, database, errors};
use crate::json;

use chrono;
use chrono::TimeDelta;
use chrono::{NaiveDateTime, NaiveDate, Local, DateTime, Duration};
use fancy_regex::Captures;
use json::JsonValue;

mod listener;
mod updater;
mod io;

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


pub struct Daemon {
    paths : ProgramPaths,
    session : Arc<Mutex<database::Session>>,
}


impl Daemon {

    /// Initialization function of the daemon only requires that the paths be specified for this runtime.
    pub fn new(paths: ProgramPaths) -> Result<Self, errors::Error> {
        
        // Initialize persistent local db.
        let session = 
            database::Session::init(&paths)
            .expect("An unexpected error occurred initializing the database.");
        
        Ok (Self { 
                    paths,
                    session : Arc::new(Mutex::new(session)),
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
        updater::run(Arc::clone(&self.session)).expect("Program does nothing if this fails");

        // Spawns a thread to continuously listen for & execute database updates
        listener::run(Arc::clone(&self.session)).expect("Program does nothing if this fails");

        Ok (())
    }


    fn parse_all_webhook_configs(&mut self) -> Result<(), errors::Error> {
        log::debug!("Starting webhook parse");
        let webhooks = read_dir(&self.paths.etc.join(PATH_WEBHOOKS_ETC))?;
        let mut webhook_path: PathBuf;

        let regex_conf = Regex::new(REGEX_CONF)?;
        let mut captures: Option<Captures<'_, String>> ;
        let mut path: String;
        let mut name: String;

        let mut webhook_config: WebhookConfigContents;
        let mut webhook_placeholders: Vec<String> = Vec::new();


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
                    period : period
                };
                
                let session = self.session.lock().expect("poisoned");

                database::write::set_webhook_parent(&session, &descriptor)?;
                database::write::set_webhook_placeholders(&session, &webhook_config.name, &webhook_placeholders)?;
                database::write::set_webhook_payload(&session, &name, &path)?;

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


