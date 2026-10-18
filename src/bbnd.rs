use core::fmt;
use std::fmt::Display;
use std::io::Read;
use std::ops::Not;
use std::str::FromStr;

use crate::errors::Error::ConfigErr;
use crate::read_dir;
use crate::PathBuf;
use crate::File;

use crate::ProgramPaths;
use crate::tokio;
use crate::dbus;
use crate::daemonize;
use crate::fancy_regex::Regex;

use crate::discord;
use crate::smtp;
use crate::database;
use crate::errors;

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


#[derive(Clone)]
pub struct WebhookDescriptor {
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
            self.name,
            self.url,
            self.webhook_type,
            self.interval,
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


pub struct BBDaemon {
    paths : ProgramPaths,
    db : Option<database::Session>,
    webhooks : Option<Vec<WebhookDescriptor>>,
}

impl BBDaemon {

    /// Initialization function of the daemon only requires that the paths be specified for this runtime.
    pub fn init(paths: ProgramPaths) -> Result<Self, errors::Error> {
        Ok (Self {
            paths,
            db : None,
            webhooks : Some(Vec::new()),
        })
    }

    pub fn run(&mut self) -> Result<(), errors::Error> {

        // Initialize connection persistent local data storage
        self.db = Some(database::Session::init(&self.paths).expect("An unexpected error occurred initializing the database."));
        
        self.parse_all_webhook_configs()?;

        // Read configs



        // validate data within persist db (bbdb.rs)
        // test connections to 
        //      SMTP (smtp.rs)
        //      Discord webhooks (discord.rs) 

        // read, validate, and store configs (bbconf.rs)

        // initialize daemon with validated data
        //      daemon publishes to session D-Bus

        // external scripts use dbus method calls to send updates
        //      bbnd updates the stored data in the database
        // on set cron intervals as defined in bbnd.d/myfile.conf, the daemon
        //   will publish updates to the webhooks and/or send emails 

        Ok (())
    }


    /// Spawns a thread that continuously updates embeds based on cron intervals
    /// and current database information
    fn update() -> Result<(), errors::Error> {
        Ok(())
    }

    
    /// Spawns a thread that continuously listens for updates via user D-BUS in 
    /// order to update database based on external signals
    fn listen() -> Result<(), errors::Error> {
        Ok(())
    }


    fn parse_all_webhook_configs(&mut self) -> Result<(), errors::Error> {

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

            log::debug!("Target string: {}", target_string);
            log::debug!("Match found: {}", captures.as_ref().is_none().not());

            // If the regex found any captures
            if captures.is_none().not() {
                // Extract the path & filename from the regex capture group
                path = captures.as_ref().unwrap().get(1).unwrap().as_str().to_string();
                name = captures.as_ref().unwrap().get(2).unwrap().as_str().to_string();

                webhook_config = parse_webhook_config(&path, &name)?;
                webhook_placeholders = parse_webhook_json_placeholders(&path, &name)?;

                let descriptor: WebhookDescriptor = WebhookDescriptor {
                    name : webhook_config.name.clone(),
                    url : webhook_config.url,
                    webhook_type : webhook_config.webhook_type,
                    interval : webhook_config.interval,
                    period : webhook_config.period,
                    placeholders: webhook_placeholders.placeholders.clone(),
                };

                self.db.as_mut().unwrap().add_webhook_parent(&descriptor)?;
                self.webhooks.as_mut().unwrap().push(descriptor);

                for placeholder in webhook_placeholders.placeholders {
                    self.db.as_mut().unwrap().add_webhook_placeholder(&webhook_config.name, &placeholder)?;
                }
            }

        }

        Ok(())
    }



}


fn parse_webhook_config(path: &String, name: &String) -> Result<WebhookConfigContents, errors::Error> {

    // Combine the path and name into a single PathBuf
    let path_full: &PathBuf = &PathBuf::from(path).join(format!("{}.conf", name));

    // Open the config file and prep a string buffer to receive it's contents
    let mut conf_file = File::open(path_full)?;
    let mut conf_str: String = String::new();

    // Write the contents of the conf file to the string buffer and parse it to a json object
    conf_file.read_to_string(&mut conf_str)?;
    let conf_json = json::parse(&conf_str)?;

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

    // Combine the path and name into a single PathBuf
    let path_full: &PathBuf = &PathBuf::from(path).join(format!("{}.json", name));

    // Open the config file and prep a string buffer to receive it's contents
    let mut json_file = File::open(path_full)?;
    let mut json_str: String = String::new();

    // Write the contents of the conf file to the string buffer and parse it to a json object
    json_file.read_to_string(&mut json_str)?;

    let re = Regex::new(REGEX_PLACEHOLDERS)?;
    let mut matches = re.captures_iter(&json_str);

    for regmatch in matches {
        placeholders.push(regmatch.unwrap().get(2).unwrap().as_str().to_string());
    }

    Ok(WebhookJsonPlaceholders {
        placeholders,
    })
}

