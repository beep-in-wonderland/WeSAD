use core::fmt;
use std::fmt::Display;
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
use crate::tokio;
use crate::dbus;
use crate::daemonize;
use crate::fancy_regex::Regex;
use crate::{Arc,Mutex,MutexGuard};
use crate::{info,error,debug,warn};
use crate::{discord, smtp, database, errors};
use crate::json;

use chrono;
use chrono::TimeDelta;
use chrono::{NaiveDateTime, NaiveDate, Local, DateTime};
use fancy_regex::Captures;
use json::JsonValue;


pub mod period;
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

#[derive(Debug, PartialEq, Clone)]
pub enum WebhookType {
    STATUS,
    ALERT,
}


impl FromStr for WebhookType {

    type Err = ();

    fn from_str(str: &str) -> Result<WebhookType, Self::Err> {

        match str.to_lowercase().trim() {
            "status" => Ok(WebhookType::STATUS),
            "alert"  => Ok(WebhookType::ALERT),
            _        => Err(()),
        }
    }

}


impl Display for WebhookType {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match self {
            WebhookType::ALERT => write!(f, "alert"),
            WebhookType::STATUS => write!(f, "status"),
        }
    }
}


#[derive(Clone)]
pub struct WebhookDescriptor {
    pub name : String,
    pub url : String,
    pub webhook_type: WebhookType,
    pub period: period::Period,
}


impl Display for WebhookDescriptor {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
            Webhook {}
            url: {}
            type: {}
            period: {}
            ", 
            self.name,
            self.url,
            self.webhook_type,
            self.period.to_string(),
        )
    }
}


struct WebhookConfigContents {
    name : String,
    url : String,
    webhook_type: WebhookType,
    interval: String,
    period: f64,
}


struct WebhookJsonPlaceholders {
    placeholders : Vec<String>,
}


pub struct BBDaemon {
    paths : ProgramPaths,
    session : Arc<Mutex<database::Session>>,
}


impl BBDaemon {

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
        updater::start(Arc::clone(&self.session)).expect("Program does nothing if this fails");

        // Spawns a thread to continuously listen for & execute database updates
        listener::start(Arc::clone(&self.session)).expect("Program does nothing if this fails");

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
        let mut webhook_placeholders: WebhookJsonPlaceholders;


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
                let period= period::Period::from_type_value(
                   &webhook_config.interval, webhook_config.period);

                log::debug!("Creating webhook descriptor for {}", name);
                let descriptor: WebhookDescriptor = WebhookDescriptor {
                    name : webhook_config.name.clone(),
                    url : webhook_config.url,
                    webhook_type : webhook_config.webhook_type,
                    period : period
                };
                
                let session = self.session.lock().expect("poisoned");

                database::write::set_webhook_parent(&session, &descriptor)?;

                for placeholder in webhook_placeholders.placeholders {
                    database::write::set_webhook_placeholder(&session, &webhook_config.name, &placeholder)?;
                }
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


