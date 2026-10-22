use core::fmt;
use std::fmt::Display;
use std::io::Read;
use std::ops::Not;
use std::str::FromStr;
use std::thread;

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
use crate::{Arc,Mutex};
use crate::mutex;
use crate::{info,error,debug,warn};

use crate::{discord, smtp, database, errors};

use crate::json;
use chrono;
use chrono::TimeDelta;
use chrono::{NaiveDateTime, NaiveDate, Local, DateTime};
use fancy_regex::Captures;
use json::JsonValue;
use mutex::MutexGuard;


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
    pub period: i64,
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
    period: i64,
}


struct WebhookJsonPlaceholders {
    placeholders : Vec<String>,
}


pub struct BBDaemonInner {
    paths : ProgramPaths,
    db : Option<database::Session>,
}


pub struct BBDaemon {
    inner: Arc<Mutex<BBDaemonInner>>,
}


impl BBDaemon {

    /// Initialization function of the daemon only requires that the paths be specified for this runtime.
    pub fn init(paths: ProgramPaths) -> Result<Self, errors::Error> {
        Ok (Self { inner : 
            Arc::new(Mutex::new( 
                BBDaemonInner {
                    paths,
                    db : None,
                }
            ))
        })
    }

    pub fn run(&mut self) -> Result<(), errors::Error> {

        // Initialize persistent local db.
        let session = Some(
                database::Session::init(&self.inner.lock().expect("poisoned").paths)
                .expect("An unexpected error occurred initializing the database."));
        self.inner.lock().expect("poisoned").db = session;

        // Read configs and store data in db
        self.parse_all_webhook_configs()?;

        // Drop all the orhpaned data from the db
        self.inner.lock().expect("poisoned")
            .db.as_mut().unwrap()
            .drop_orphaned_values()?;

        // Spawn the update_loop thread with an Arc clone of the self.inner mutex to share data between threads
        // Will continuously send out webhook updates as defined by the webhook configs
        self.spawn_update_loop()?;

        // Listen on the D-Bus for updates to send to the shared BBDaemonInner data
        self.listen();

        // external scripts use dbus method calls to send updates
        //      bbnd updates the stored data in the database
        // on set cron intervals as defined in bbnd.d/myfile.conf, the daemon
        //   will publish updates to the webhooks and/or send emails 

        Ok (())
    }


    fn parse_all_webhook_configs(&mut self) -> Result<(), errors::Error> {
        log::debug!("Starting webhook parse");
        let webhooks = read_dir(&self.inner.lock().expect("poisoned").paths.etc.join(PATH_WEBHOOKS_ETC))?;
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

            log::debug!("Target string for config: {}", target_string);
            log::debug!("Match found: {}", captures.as_ref().is_none().not());

            // If the regex found any captures
            if captures.is_none().not() {
                // Extract the path & filename from the regex capture group
                path = captures.as_ref().unwrap().get(1).unwrap().as_str().to_string();
                name = captures.as_ref().unwrap().get(2).unwrap().as_str().to_string();

                webhook_config = parse_webhook_config(&path, &name)?;
                webhook_placeholders = parse_webhook_json_placeholders(&path, &name)?;

                // Calculate the period in seconds based on `interval` and `period`
                let period: i64 = match webhook_config.interval.to_lowercase().trim() {
                    "seconds" => webhook_config.period,
                    "minutes" => webhook_config.period*60, // Convert minutes to seconds
                    "hours" => webhook_config.period*60*60, // Convert hours to seconds
                    _ => 0
                };

                log::debug!("Creating webhook descriptor for {}", name);
                let descriptor: WebhookDescriptor = WebhookDescriptor {
                    name : webhook_config.name.clone(),
                    url : webhook_config.url,
                    webhook_type : webhook_config.webhook_type,
                    period : period
                };
                
                self.inner.lock().expect("poisoned")
                    .db.as_mut().unwrap().set_webhook_parent(&descriptor)?;

                for placeholder in webhook_placeholders.placeholders {
                    self.inner.lock().expect("poisoned").db.as_mut().unwrap()
                        .set_webhook_placeholder(&webhook_config.name, &placeholder)?;
                }
            }

        }

        Ok(())
    }

    fn spawn_update_loop(&self) -> Result<(), errors::Error> {

        let mut thread_bbnd = Arc::clone(&self.inner);

        thread::spawn( move || {
            update_loop(&mut thread_bbnd);
        });
        debug!("Update thread spawned successfully");

        Ok(())
    }


    /// Continuously listens for updates via user D-BUS in order to update the shared BBDaemonInner mutex
    fn listen(&self) -> () {

        // listen() should publish this process on the D-Bus, then iterate through the following procedures:
        // 1. Append any incoming signals to a queue
        // 
        // 2. Process the request as either a webhook status, or webhook alert
        // 3. Validate that the target of the data is correct.
        //    IF: webhook status:
        //          Lock the BBDaemonInner mutex, write to the db, then release the guard.
        //    IF: webhook alert:
        //          Send the alert data to the discord module and continue listening

        loop {
            break;
        }
    }

}


fn parse_webhook_config(path: &String, name: &String) -> Result<WebhookConfigContents, errors::Error> {

    // Open the config file
    let conf_str: &String = &open_as_string(path, name, ".conf")?;

    let conf_json = json::parse(conf_str)?;

    validate_config_lines(&conf_json, &name)?;

    Ok(WebhookConfigContents { 
        name: name.clone(),
        url : conf_json["url"].to_string(), 
        webhook_type : WebhookType::from_str(conf_json["type"].as_str().unwrap()).unwrap(), 
        interval : conf_json["interval"].to_string(),
        period : conf_json["period"].as_i64().unwrap(),
    })
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


fn parse_webhook_json_placeholders(path: &String, name: &String) -> Result<WebhookJsonPlaceholders, errors::Error> {

    let mut placeholders = Vec::new();

    // Open the json template embed file
    let json_str: &String = &open_as_string(path, name, ".json")?;

    let re = Regex::new(REGEX_PLACEHOLDERS)?;
    let mut matches = re.captures_iter(json_str);

    for regmatch in matches {
        placeholders.push(regmatch.unwrap().get(2).unwrap().as_str().to_string());
    }

    Ok(WebhookJsonPlaceholders {
        placeholders,
    })
}


fn open_as_string(path: &String, name: &String, ext: &str) -> Result<String, errors::Error> {

     // Combine the path and name into a single PathBuf
    let path_full: &PathBuf = &PathBuf::from(path).join(format!("{}{}",name, ext));

    // Open the config file and prep a string buffer to receive it's contents
    let mut file = File::open(path_full)?;
    let mut str_buf: String = String::new();

    // Write the contents of the conf file to the string buffer and parse it to a json object
    file.read_to_string(&mut str_buf)?;
    Ok(str_buf)
}


/// Continuously updates the `status` webhooks in the `BBDaemonInner` struct based on their individual
/// update interval types and periods.
fn update_loop(data: &mut Arc<Mutex<BBDaemonInner>>) -> () {
    // Each iteration, the loop should:
    // 1. Lock the mutex
    // 2. Fetch all status type webhooks
    // 3. Compare the `last` + `period` fields to the current DateTime
    // 4. Execute any updates where necessary, update the `last` fields
    // 5. Find the minimum seconds of the `last` fields for the next update
    // 6. Copy the data into a local variable `sleep_duration`
    // 7. Release the mutex
    // 8. Put the thread to sleep for `sleep_duration` seconds

    let mut webhooks: &Vec<WebhookDescriptor>;

    let mut period: f64;
    let mut last: DateTime<Local>;
    let mut next: DateTime<Local>;
    let mut time_delta: TimeDelta;

    loop {

        // 1. Lock the mutex
        let mut m = data.lock().expect("poisoned");

        // 2. Fetch the webhooks as a list sorted by next execution
        let webhooks = m.db.as_ref().unwrap().get_webhook_status_data().unwrap();

        for webhook in webhooks {
            // TODO
        }


        drop(m);
        break;
    }
}