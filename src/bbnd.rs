use core::fmt;
use std::fmt::Display;
use std::io::Read;
use std::ops::Not;
use std::str::FromStr;
use std::thread;

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
use chrono::NaiveDate;
use fancy_regex::Captures;
use json::JsonValue;


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


pub struct WebhookDescriptor {
    pub inner: Arc<Mutex<WebhookDescriptorInner>>,
}


#[derive(Clone)]
pub struct WebhookDescriptorInner {
    pub path: String,
    pub name : String,
    pub url : String,
    pub webhook_type: WebhookType,
    pub interval: String,
    pub period: i64,
    pub placeholders: Vec<String>,
}


impl Display for WebhookDescriptor {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
            Webhook {}
            url: {}
            type: {}
            interval: {}
            period: {}
            ", 
            self.inner.lock().expect("poisoned").name,
            self.inner.lock().expect("poisoned").url,
            self.inner.lock().expect("poisoned").webhook_type,
            self.inner.lock().expect("poisoned").interval,
            self.inner.lock().expect("poisoned").period.to_string(),
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


pub struct BBDaemon {
    inner: Arc<Mutex<BBDaemonInner>>,
}

pub struct BBDaemonInner {
    paths : ProgramPaths,
    db : Option<database::Session>,
    webhooks : Option<Vec<WebhookDescriptor>>,
}

impl BBDaemon {

    /// Initialization function of the daemon only requires that the paths be specified for this runtime.
    pub fn init(paths: ProgramPaths) -> Result<Self, errors::Error> {
        Ok (Self { inner : 
            Arc::new(Mutex::new( 
                BBDaemonInner {
                    paths,
                    db : None,
                    webhooks : Some(Vec::new()),
                }
            ))
        })
    }

    pub fn run(&mut self) -> Result<(), errors::Error> {

        // Initialize connection persistent local data storage
        let session = Some(
                database::Session::init(&self.inner.lock().expect("poisoned").paths)
                .expect("An unexpected error occurred initializing the database."));
        self.inner.lock().expect("poisoned").db = session;

        debug!("Database initialization complete");
        
        // Read configs and store data in self
        self.parse_all_webhook_configs()?;

        // Drop all the orhpaned values from the db
        self.inner.lock().expect("poisoned")
            .db.as_mut().unwrap()
            .drop_orphaned_values()?;


        // validate data within persist db (bbdb.rs)
        // test connections to 
        //      SMTP (smtp.rs)
        //      Discord webhooks (discord.rs) 

        // read, validate, and store configs (bbconf.rs)

        // external scripts use dbus method calls to send updates
        //      bbnd updates the stored data in the database
        // on set cron intervals as defined in bbnd.d/myfile.conf, the daemon
        //   will publish updates to the webhooks and/or send emails 

        Ok (())
    }


    /// Spawns a thread that continuously updates embeds based on cron intervals
    /// and current database information
    fn update_loop(&self) -> Result<(), errors::Error> {
        
        // while True:
        //  Iterate over & execute first round of webhook updates
        //  If a value is null in db, use a default value e.g. NULL or :black_circle:

        //  Find the time in seconds between now and the next scheduled update from all the status webhooks
        //  let mut sleep_interval: &f64 = next_update_shortest()?;
        //  sleep(sleep_interval)

        Ok(())
    }

    
    /// Spawns a thread that continuously listens for updates via user D-BUS in 
    /// order to update database based on external signals
    fn listen() -> Result<(), errors::Error> {
        
        // Figure out D-Bus shit

        Ok(())
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

                log::debug!("Creating inner webhook descriptor for {}", name);
                let descriptor_inner: Arc<Mutex<WebhookDescriptorInner>> = Arc::new(Mutex::new(WebhookDescriptorInner {
                    path,
                    name : webhook_config.name.clone(),
                    url : webhook_config.url,
                    webhook_type : webhook_config.webhook_type,
                    interval : webhook_config.interval,
                    period : webhook_config.period,
                    placeholders: webhook_placeholders.placeholders.clone(),
                }));

                let descriptor = WebhookDescriptor { inner: descriptor_inner };

                log::debug!("Pushing webhook descriptor to self {}", name);
                
                self.inner.lock().expect("poisoned")
                    .db.as_mut().unwrap().add_webhook_parent(&descriptor)?;

                self.inner.lock().expect("poisoned")
                    .webhooks.as_mut().unwrap().push(descriptor);

                for placeholder in webhook_placeholders.placeholders {
                    self.inner.lock().expect("poisoned").db.as_mut().unwrap()
                        .add_webhook_placeholder(&webhook_config.name, &placeholder)?;
                }
            }

        }

        Ok(())
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

